use crate::cpu::WorkBudget;
use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::lair_contact::{distance_to_interval, travel_units};
use crate::game::geometry::MovementSegment;
use crate::game::types::HALF_FOOTPRINT;
use crate::game::{PickupKind, Position, Tick};

use super::continuation_contact::ContinuationContact as TeleporterContact;
use super::motion::Motion;
use super::safety::{check_segments_with_grace, static_segment_clear};
use super::teleporter_step::{take, usize_to_work};
use super::teleporter_step_geometry::{
    Region, charge_segment_scans, charge_static_scans, charged_region_cells, contact_failure,
    distance_to_cell, intersects, max_travel, next_tick, one_tick, partial_endpoints,
    precharge_outcome_storage, previous_tick, push_bounded, region_around, reject_pits,
    reject_storm_region, visible_alive,
};
use super::teleporter_step_types::{OutcomeRegion, TeleporterStepPhase, TeleporterStepRequest};
use super::validation::Context;
use super::{RouteFailure, RouteMode};

pub(super) struct ProofInput<'a> {
    pub(super) context: &'a Context<'a>,
    pub(super) forecast: &'a HazardForecast,
    pub(super) request: TeleporterStepRequest,
    pub(super) phase: TeleporterStepPhase,
    pub(super) safe_until: Tick,
    pub(super) nominal: &'a [(Tick, Motion, Vec<MovementSegment>)],
    pub(super) allow_distant_teleporters: bool,
    pub(super) work: &'a mut WorkBudget,
}

pub(super) struct ProofOutcome {
    pub(super) nominal_positions: Vec<Position>,
    pub(super) collecting_positions: Vec<Position>,
    pub(super) clipped_positions: Vec<Position>,
    pub(super) landing_regions: Vec<OutcomeRegion>,
}

pub(super) fn prove(mut input: ProofInput<'_>) -> Result<ProofOutcome, RouteFailure> {
    reject_reachable_pickups(&mut input)?;
    reject_contested_target(&mut input)?;
    let contact =
        TeleporterContact::new(input.context.observation, input.work).map_err(contact_failure)?;
    let (possible_capacity, collecting_capacity, landing_capacity) = precharge_outcome_storage(
        input.nominal.len(),
        input.phase == TeleporterStepPhase::Collect,
        input.work,
    )?;
    let mut possible = Vec::with_capacity(possible_capacity);
    take(input.work, usize_to_work(input.nominal.len())?)?;
    let mut nominal_positions = Vec::with_capacity(input.nominal.len());
    let mut collecting_positions = Vec::with_capacity(collecting_capacity);
    let mut landing_regions = Vec::with_capacity(landing_capacity);
    for (start, motion, segments) in input.nominal {
        take(input.work, 1)?;
        let first_wait = next_tick(input.context.observation.parts().tick)?;
        if first_wait < *start {
            let waiting = [MovementSegment::new(
                input.context.actor.position,
                input.context.actor.position,
            )];
            charge_segment_scans(input.forecast, &waiting, input.work)?;
            check_segments_with_grace(
                input.forecast,
                &waiting,
                TickWindow {
                    start: first_wait,
                    end: *start,
                },
                RouteMode::CertifiedOnly,
                None,
                None,
            )?;
            if !contact
                .stationary_is_clear_until(
                    input.context.actor.position,
                    previous_tick(*start)?,
                    input.work,
                )
                .map_err(contact_failure)?
            {
                return Err(RouteFailure::NoRoute);
            }
        }
        take(input.work, usize_to_work(segments.len())?)?;
        validate_named_pickup_sweep(&mut input, segments)?;
        charge_segment_scans(input.forecast, segments, input.work)?;
        charge_static_scans(input.context.observation, segments, input.work)?;
        if !static_segment_clear(input.context.observation, segments) {
            return Err(RouteFailure::NoRoute);
        }
        check_segments_with_grace(
            input.forecast,
            segments,
            one_tick(*start)?,
            RouteMode::CertifiedOnly,
            None,
            None,
        )?;
        if !contact
            .sweep_is_clear(*start, segments, input.work)
            .map_err(contact_failure)?
        {
            return Err(RouteFailure::NoRoute);
        }
        let endpoints = partial_endpoints(input.context.actor.position, segments, input.work)?;
        for position in endpoints {
            if input.phase == TeleporterStepPhase::Approach
                && position.cell() == Some(input.request.target)
            {
                return Err(RouteFailure::NoRoute);
            }
            let stopped = [MovementSegment::new(position, position)];
            charge_segment_scans(input.forecast, &stopped, input.work)?;
            check_segments_with_grace(
                input.forecast,
                &stopped,
                TickWindow {
                    start: next_tick(*start)?,
                    end: input.safe_until,
                },
                RouteMode::CertifiedOnly,
                None,
                None,
            )?;
            if !contact
                .stationary_is_clear_until(position, previous_tick(input.safe_until)?, input.work)
                .map_err(contact_failure)?
            {
                return Err(RouteFailure::NoRoute);
            }
            reject_pits(input.context, &stopped, input.work)?;
            if input.phase == TeleporterStepPhase::Collect
                && position.cell() == Some(input.request.target)
            {
                push_bounded(&mut collecting_positions, position)?;
            }
            push_bounded(&mut possible, position)?;
        }
        if input.phase == TeleporterStepPhase::Collect {
            prove_teleport_outcomes(
                &mut input,
                &contact,
                *start,
                motion.position,
                &mut landing_regions,
            )?;
        }
        nominal_positions.push(motion.position);
    }
    Ok(ProofOutcome {
        nominal_positions,
        collecting_positions,
        clipped_positions: possible,
        landing_regions,
    })
}

fn validate_named_pickup_sweep(
    input: &mut ProofInput<'_>,
    segments: &[MovementSegment],
) -> Result<(), RouteFailure> {
    for pickup in &input.context.observation.parts().pickups {
        take(
            input.work,
            usize_to_work(
                segments
                    .len()
                    .checked_add(1)
                    .ok_or(RouteFailure::Capacity)?,
            )?,
        )?;
        if !segments
            .iter()
            .any(|segment| segment.overlaps_cell(pickup.cell))
        {
            continue;
        }
        if pickup.kind != PickupKind::Teleporter || pickup.cell != input.request.target {
            return Err(RouteFailure::Unsupported);
        }
        if input.phase == TeleporterStepPhase::Approach
            && segments
                .iter()
                .any(|segment| segment.end().cell() == Some(pickup.cell))
        {
            return Err(RouteFailure::NoRoute);
        }
    }
    reject_pits(input.context, segments, input.work)
}

fn reject_reachable_pickups(input: &mut ProofInput<'_>) -> Result<(), RouteFailure> {
    // A successful swap only permutes actor positions. Bounding every living
    // basis origin, including the root's, therefore also covers the partner
    // that may inherit that origin after collecting the selected Teleporter.
    let ticks = input.safe_until.0 - input.context.observation.parts().tick.0;
    let travel = max_travel(ticks)?;
    let parts = input.context.observation.parts();
    let scans = parts
        .pickups
        .len()
        .checked_mul(
            parts
                .actors
                .len()
                .checked_add(1)
                .ok_or(RouteFailure::Capacity)?,
        )
        .ok_or(RouteFailure::Capacity)?;
    take(input.work, usize_to_work(scans)?)?;
    for pickup in input
        .context
        .observation
        .parts()
        .pickups
        .iter()
        .filter(|pickup| {
            if input.allow_distant_teleporters {
                pickup.cell != input.request.target
            } else {
                pickup.kind != PickupKind::Teleporter
            }
        })
    {
        for actor in input
            .context
            .observation
            .parts()
            .actors
            .iter()
            .filter_map(visible_alive)
        {
            take(input.work, 1)?;
            if distance_to_cell(actor.position, pickup.cell) <= travel {
                return Err(RouteFailure::Unsupported);
            }
        }
    }
    Ok(())
}

fn reject_contested_target(input: &mut ProofInput<'_>) -> Result<(), RouteFailure> {
    let basis = input.context.observation.parts().tick;
    take(
        input.work,
        usize_to_work(input.context.observation.parts().actors.len())?,
    )?;
    for actor in input
        .context
        .observation
        .parts()
        .actors
        .iter()
        .filter_map(visible_alive)
    {
        if actor.seat == input.context.observation.parts().recipient {
            continue;
        }
        take(input.work, 1)?;
        if distance_to_cell(actor.position, input.request.target)
            <= max_travel(input.safe_until.0 - basis.0)?
        {
            return Err(RouteFailure::NoRoute);
        }
    }
    Ok(())
}

fn prove_teleport_outcomes(
    input: &mut ProofInput<'_>,
    contact: &TeleporterContact,
    collect_tick: Tick,
    root_at_pickup: Position,
    landing_regions: &mut Vec<OutcomeRegion>,
) -> Result<(), RouteFailure> {
    let elapsed = collect_tick.0 - input.context.observation.parts().tick.0;
    take(
        input.work,
        usize_to_work(input.context.observation.parts().actors.len())?,
    )?;
    for partner in input
        .context
        .observation
        .parts()
        .actors
        .iter()
        .filter_map(visible_alive)
    {
        if partner.seat == input.context.observation.parts().recipient {
            continue;
        }
        take(input.work, 1)?;
        let landing = region_around(partner.position, max_travel(elapsed)?)?;
        validate_landing_region(input, contact, landing, collect_tick, input.safe_until)?;
        push_bounded(
            landing_regions,
            OutcomeRegion {
                min: landing.min,
                max: landing.max,
            },
        )?;

        // The partner keeps its held input, but only its collection endpoint can
        // interact with the root before the root's mandatory Stop.
        let partner_endpoint = Region {
            min: root_at_pickup,
            max: root_at_pickup,
        };
        validate_landing_region(
            input,
            contact,
            partner_endpoint,
            collect_tick,
            next_tick(collect_tick)?,
        )?;
    }
    Ok(())
}

fn validate_landing_region(
    input: &mut ProofInput<'_>,
    contact: &TeleporterContact,
    region: Region,
    begins: Tick,
    ends: Tick,
) -> Result<(), RouteFailure> {
    let cells = charged_region_cells(region, input.forecast.dimensions(), input.work)?;
    if let Some(pits) = input.context.observation.spike_pits() {
        let scans = cells
            .len()
            .checked_mul(pits.pits().len())
            .ok_or(RouteFailure::Capacity)?;
        take(input.work, usize_to_work(scans)?)?;
    }
    for cell in cells {
        take(input.work, 1)?;
        if input
            .context
            .observation
            .spike_pits()
            .is_some_and(|pits| pits.pits().iter().any(|pit| pit.cell == cell))
        {
            return Err(RouteFailure::NoRoute);
        }
        let forecast = input
            .forecast
            .cell(cell)
            .ok_or(RouteFailure::InvalidRequest)?;
        let scans = forecast
            .lethal
            .len()
            .checked_add(forecast.uncertainty.len())
            .ok_or(RouteFailure::Capacity)?;
        take(input.work, usize_to_work(scans)?)?;
        let window = TickWindow {
            start: begins,
            end: ends,
        };
        if forecast
            .lethal
            .iter()
            .any(|hazard| intersects(*hazard, window))
            || forecast
                .uncertainty
                .iter()
                .any(|hazard| intersects(hazard.window, window))
        {
            return Err(RouteFailure::NoRoute);
        }
    }
    reject_storm_region(input.context, region, begins, ends, input.work)?;
    let last = previous_tick(ends)?;
    if let TeleporterContact::Enabled(envelope) = contact {
        for source in envelope.sources() {
            take(input.work, 1)?;
            if source.begins_at > last {
                continue;
            }
            let travel = travel_units(source.begins_at, last).map_err(contact_failure)?;
            let contact = i64::from(HALF_FOOTPRINT) * 2 - 1;
            let dx = distance_to_interval(
                i64::from(source.origin.x),
                i64::from(region.min.x) - contact,
                i64::from(region.max.x) + contact,
            );
            let dy = distance_to_interval(
                i64::from(source.origin.y),
                i64::from(region.min.y) - contact,
                i64::from(region.max.y) + contact,
            );
            if dx.saturating_add(dy) <= travel {
                return Err(RouteFailure::NoRoute);
            }
        }
    }
    Ok(())
}
