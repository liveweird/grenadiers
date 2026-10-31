//#region \0rolldown/runtime.js
var __commonJSMin = (cb, mod) => () => (mod || (cb((mod = { exports: {} }).exports, mod, mod.exports), cb = null), mod.exports);
//#endregion
//#region \0grenadiers-validator-entry/cpu-debug.cjs
var require_cpu_debug = /* @__PURE__ */ __commonJSMin(((exports, module) => {
	module.exports = validate20;
	module.exports.default = validate20;
	var schema32 = {
		"type": "string",
		"enum": [
			"one",
			"two",
			"three",
			"four"
		]
	};
	var pattern4 = /* @__PURE__ */ new RegExp("^[0-9a-f]{32}$", "u");
	var schema33 = { "oneOf": [{
		"type": "object",
		"additionalProperties": false,
		"required": ["seat", "status"],
		"properties": {
			"seat": { "$ref": "#/$defs/seat" },
			"status": { "const": "unavailable" }
		}
	}, {
		"type": "object",
		"additionalProperties": false,
		"required": [
			"seat",
			"status",
			"observedTick",
			"objective",
			"target",
			"approach",
			"phase",
			"switchReason"
		],
		"properties": {
			"seat": { "$ref": "#/$defs/seat" },
			"status": { "const": "available" },
			"observedTick": {
				"type": "integer",
				"minimum": 0,
				"maximum": 9007199254740991
			},
			"objective": {
				"type": "string",
				"enum": [
					"collect",
					"farm",
					"attack",
					"explore",
					"survive",
					"wait"
				]
			},
			"target": { "anyOf": [{ "$ref": "#/$defs/target" }, { "type": "null" }] },
			"approach": { "anyOf": [{ "$ref": "#/$defs/cell" }, { "type": "null" }] },
			"phase": {
				"type": "string",
				"enum": [
					"approach",
					"plant",
					"escape",
					"reengage",
					"evade",
					"waiting"
				]
			},
			"switchReason": {
				"type": "string",
				"enum": [
					"initial",
					"completed",
					"targetInvalid",
					"visibilityLost",
					"betterOpportunity",
					"stalled",
					"noTargets",
					"safetyInterrupt",
					"resumed"
				]
			}
		}
	}] };
	var schema36 = { "oneOf": [
		{
			"type": "object",
			"additionalProperties": false,
			"required": [
				"type",
				"cell",
				"kind"
			],
			"properties": {
				"type": { "const": "pickup" },
				"cell": { "$ref": "#/$defs/cell" },
				"kind": {
					"type": "string",
					"enum": [
						"bombCapacity",
						"blastRange",
						"movementSpeed",
						"kick",
						"punch",
						"bracelet",
						"wallPass",
						"heart",
						"remoteBomb",
						"piercingBomb",
						"powerBomb",
						"landmine",
						"shield",
						"stopwatch",
						"invisibilityCloak",
						"goldenFlame",
						"shortFuse",
						"diarrhea",
						"slowFoot",
						"shortFlame",
						"longFuse",
						"reverseControls",
						"invisibleBombs",
						"oilBarrel",
						"teleporter"
					]
				}
			}
		},
		{
			"type": "object",
			"additionalProperties": false,
			"required": ["type", "cell"],
			"properties": {
				"type": { "enum": ["block", "tile"] },
				"cell": { "$ref": "#/$defs/cell" }
			}
		},
		{
			"type": "object",
			"additionalProperties": false,
			"required": ["type", "seat"],
			"properties": {
				"type": { "const": "opponent" },
				"seat": { "$ref": "#/$defs/seat" }
			}
		}
	] };
	function validate22(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate22.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/oneOf/0/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.cell === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/oneOf/0/required",
					keyword: "required",
					params: { missingProperty: "cell" },
					message: "must have required property 'cell'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.kind === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/oneOf/0/required",
					keyword: "required",
					params: { missingProperty: "kind" },
					message: "must have required property 'kind'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "cell" || key0 === "kind")) {
				const err3 = {
					instancePath,
					schemaPath: "#/oneOf/0/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.type !== void 0) {
				if ("pickup" !== data.type) {
					const err4 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/oneOf/0/properties/type/const",
						keyword: "const",
						params: { allowedValue: "pickup" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.cell !== void 0) {
				let data1 = data.cell;
				if (data1 && typeof data1 == "object" && !Array.isArray(data1)) {
					if (data1.column === void 0) {
						const err5 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/cell/required",
							keyword: "required",
							params: { missingProperty: "column" },
							message: "must have required property 'column'"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1.row === void 0) {
						const err6 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/cell/required",
							keyword: "required",
							params: { missingProperty: "row" },
							message: "must have required property 'row'"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					for (const key1 in data1) if (!(key1 === "column" || key1 === "row")) {
						const err7 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/cell/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data1.column !== void 0) {
						let data2 = data1.column;
						if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
							const err8 = {
								instancePath: instancePath + "/cell/column",
								schemaPath: "#/$defs/cell/properties/column/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err8];
							else vErrors.push(err8);
							errors++;
						}
						if (typeof data2 == "number" && isFinite(data2)) {
							if (data2 > 14 || isNaN(data2)) {
								const err9 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/cell/properties/column/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 14
									},
									message: "must be <= 14"
								};
								if (vErrors === null) vErrors = [err9];
								else vErrors.push(err9);
								errors++;
							}
							if (data2 < 0 || isNaN(data2)) {
								const err10 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/cell/properties/column/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err10];
								else vErrors.push(err10);
								errors++;
							}
						}
					}
					if (data1.row !== void 0) {
						let data3 = data1.row;
						if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3) && isFinite(data3))) {
							const err11 = {
								instancePath: instancePath + "/cell/row",
								schemaPath: "#/$defs/cell/properties/row/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err11];
							else vErrors.push(err11);
							errors++;
						}
						if (typeof data3 == "number" && isFinite(data3)) {
							if (data3 > 12 || isNaN(data3)) {
								const err12 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/cell/properties/row/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 12
									},
									message: "must be <= 12"
								};
								if (vErrors === null) vErrors = [err12];
								else vErrors.push(err12);
								errors++;
							}
							if (data3 < 0 || isNaN(data3)) {
								const err13 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/cell/properties/row/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err13];
								else vErrors.push(err13);
								errors++;
							}
						}
					}
				} else {
					const err14 = {
						instancePath: instancePath + "/cell",
						schemaPath: "#/$defs/cell/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err14];
					else vErrors.push(err14);
					errors++;
				}
			}
			if (data.kind !== void 0) {
				let data4 = data.kind;
				if (typeof data4 !== "string") {
					const err15 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/oneOf/0/properties/kind/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err15];
					else vErrors.push(err15);
					errors++;
				}
				if (!(data4 === "bombCapacity" || data4 === "blastRange" || data4 === "movementSpeed" || data4 === "kick" || data4 === "punch" || data4 === "bracelet" || data4 === "wallPass" || data4 === "heart" || data4 === "remoteBomb" || data4 === "piercingBomb" || data4 === "powerBomb" || data4 === "landmine" || data4 === "shield" || data4 === "stopwatch" || data4 === "invisibilityCloak" || data4 === "goldenFlame" || data4 === "shortFuse" || data4 === "diarrhea" || data4 === "slowFoot" || data4 === "shortFlame" || data4 === "longFuse" || data4 === "reverseControls" || data4 === "invisibleBombs" || data4 === "oilBarrel" || data4 === "teleporter")) {
					const err16 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/oneOf/0/properties/kind/enum",
						keyword: "enum",
						params: { allowedValues: schema36.oneOf[0].properties.kind.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err16];
					else vErrors.push(err16);
					errors++;
				}
			}
		} else {
			const err17 = {
				instancePath,
				schemaPath: "#/oneOf/0/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err17];
			else vErrors.push(err17);
			errors++;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
			var props0 = true;
		}
		const _errs15 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err18 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err18];
				else vErrors.push(err18);
				errors++;
			}
			if (data.cell === void 0) {
				const err19 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "cell" },
					message: "must have required property 'cell'"
				};
				if (vErrors === null) vErrors = [err19];
				else vErrors.push(err19);
				errors++;
			}
			for (const key2 in data) if (!(key2 === "type" || key2 === "cell")) {
				const err20 = {
					instancePath,
					schemaPath: "#/oneOf/1/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key2 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err20];
				else vErrors.push(err20);
				errors++;
			}
			if (data.type !== void 0) {
				let data5 = data.type;
				if (!(data5 === "block" || data5 === "tile")) {
					const err21 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/oneOf/1/properties/type/enum",
						keyword: "enum",
						params: { allowedValues: schema36.oneOf[1].properties.type.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err21];
					else vErrors.push(err21);
					errors++;
				}
			}
			if (data.cell !== void 0) {
				let data6 = data.cell;
				if (data6 && typeof data6 == "object" && !Array.isArray(data6)) {
					if (data6.column === void 0) {
						const err22 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/cell/required",
							keyword: "required",
							params: { missingProperty: "column" },
							message: "must have required property 'column'"
						};
						if (vErrors === null) vErrors = [err22];
						else vErrors.push(err22);
						errors++;
					}
					if (data6.row === void 0) {
						const err23 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/cell/required",
							keyword: "required",
							params: { missingProperty: "row" },
							message: "must have required property 'row'"
						};
						if (vErrors === null) vErrors = [err23];
						else vErrors.push(err23);
						errors++;
					}
					for (const key3 in data6) if (!(key3 === "column" || key3 === "row")) {
						const err24 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/cell/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key3 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err24];
						else vErrors.push(err24);
						errors++;
					}
					if (data6.column !== void 0) {
						let data7 = data6.column;
						if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7) && isFinite(data7))) {
							const err25 = {
								instancePath: instancePath + "/cell/column",
								schemaPath: "#/$defs/cell/properties/column/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err25];
							else vErrors.push(err25);
							errors++;
						}
						if (typeof data7 == "number" && isFinite(data7)) {
							if (data7 > 14 || isNaN(data7)) {
								const err26 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/cell/properties/column/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 14
									},
									message: "must be <= 14"
								};
								if (vErrors === null) vErrors = [err26];
								else vErrors.push(err26);
								errors++;
							}
							if (data7 < 0 || isNaN(data7)) {
								const err27 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/cell/properties/column/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err27];
								else vErrors.push(err27);
								errors++;
							}
						}
					}
					if (data6.row !== void 0) {
						let data8 = data6.row;
						if (!(typeof data8 == "number" && !(data8 % 1) && !isNaN(data8) && isFinite(data8))) {
							const err28 = {
								instancePath: instancePath + "/cell/row",
								schemaPath: "#/$defs/cell/properties/row/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err28];
							else vErrors.push(err28);
							errors++;
						}
						if (typeof data8 == "number" && isFinite(data8)) {
							if (data8 > 12 || isNaN(data8)) {
								const err29 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/cell/properties/row/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 12
									},
									message: "must be <= 12"
								};
								if (vErrors === null) vErrors = [err29];
								else vErrors.push(err29);
								errors++;
							}
							if (data8 < 0 || isNaN(data8)) {
								const err30 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/cell/properties/row/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err30];
								else vErrors.push(err30);
								errors++;
							}
						}
					}
				} else {
					const err31 = {
						instancePath: instancePath + "/cell",
						schemaPath: "#/$defs/cell/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err31];
					else vErrors.push(err31);
					errors++;
				}
			}
		} else {
			const err32 = {
				instancePath,
				schemaPath: "#/oneOf/1/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err32];
			else vErrors.push(err32);
			errors++;
		}
		var _valid0 = _errs15 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else {
			if (_valid0) {
				valid0 = true;
				passing0 = 1;
				if (props0 !== true) props0 = true;
			}
			const _errs27 = errors;
			if (data && typeof data == "object" && !Array.isArray(data)) {
				if (data.type === void 0) {
					const err33 = {
						instancePath,
						schemaPath: "#/oneOf/2/required",
						keyword: "required",
						params: { missingProperty: "type" },
						message: "must have required property 'type'"
					};
					if (vErrors === null) vErrors = [err33];
					else vErrors.push(err33);
					errors++;
				}
				if (data.seat === void 0) {
					const err34 = {
						instancePath,
						schemaPath: "#/oneOf/2/required",
						keyword: "required",
						params: { missingProperty: "seat" },
						message: "must have required property 'seat'"
					};
					if (vErrors === null) vErrors = [err34];
					else vErrors.push(err34);
					errors++;
				}
				for (const key4 in data) if (!(key4 === "type" || key4 === "seat")) {
					const err35 = {
						instancePath,
						schemaPath: "#/oneOf/2/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key4 },
						message: "must NOT have additional properties"
					};
					if (vErrors === null) vErrors = [err35];
					else vErrors.push(err35);
					errors++;
				}
				if (data.type !== void 0) {
					if ("opponent" !== data.type) {
						const err36 = {
							instancePath: instancePath + "/type",
							schemaPath: "#/oneOf/2/properties/type/const",
							keyword: "const",
							params: { allowedValue: "opponent" },
							message: "must be equal to constant"
						};
						if (vErrors === null) vErrors = [err36];
						else vErrors.push(err36);
						errors++;
					}
				}
				if (data.seat !== void 0) {
					let data10 = data.seat;
					if (typeof data10 !== "string") {
						const err37 = {
							instancePath: instancePath + "/seat",
							schemaPath: "#/$defs/seat/type",
							keyword: "type",
							params: { type: "string" },
							message: "must be string"
						};
						if (vErrors === null) vErrors = [err37];
						else vErrors.push(err37);
						errors++;
					}
					if (!(data10 === "one" || data10 === "two" || data10 === "three" || data10 === "four")) {
						const err38 = {
							instancePath: instancePath + "/seat",
							schemaPath: "#/$defs/seat/enum",
							keyword: "enum",
							params: { allowedValues: schema32.enum },
							message: "must be equal to one of the allowed values"
						};
						if (vErrors === null) vErrors = [err38];
						else vErrors.push(err38);
						errors++;
					}
				}
			} else {
				const err39 = {
					instancePath,
					schemaPath: "#/oneOf/2/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err39];
				else vErrors.push(err39);
				errors++;
			}
			var _valid0 = _errs27 === errors;
			if (_valid0 && valid0) {
				valid0 = false;
				passing0 = [passing0, 2];
			} else if (_valid0) {
				valid0 = true;
				passing0 = 2;
				if (props0 !== true) props0 = true;
			}
		}
		if (!valid0) {
			const err40 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err40];
			else vErrors.push(err40);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate22.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate22.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	function validate21(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate21.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.seat === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/oneOf/0/required",
					keyword: "required",
					params: { missingProperty: "seat" },
					message: "must have required property 'seat'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.status === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/oneOf/0/required",
					keyword: "required",
					params: { missingProperty: "status" },
					message: "must have required property 'status'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "seat" || key0 === "status")) {
				const err2 = {
					instancePath,
					schemaPath: "#/oneOf/0/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.seat !== void 0) {
				let data0 = data.seat;
				if (typeof data0 !== "string") {
					const err3 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/seat/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
				if (!(data0 === "one" || data0 === "two" || data0 === "three" || data0 === "four")) {
					const err4 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/seat/enum",
						keyword: "enum",
						params: { allowedValues: schema32.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.status !== void 0) {
				if ("unavailable" !== data.status) {
					const err5 = {
						instancePath: instancePath + "/status",
						schemaPath: "#/oneOf/0/properties/status/const",
						keyword: "const",
						params: { allowedValue: "unavailable" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
		} else {
			const err6 = {
				instancePath,
				schemaPath: "#/oneOf/0/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err6];
			else vErrors.push(err6);
			errors++;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
			var props0 = true;
		}
		const _errs8 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.seat === void 0) {
				const err7 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "seat" },
					message: "must have required property 'seat'"
				};
				if (vErrors === null) vErrors = [err7];
				else vErrors.push(err7);
				errors++;
			}
			if (data.status === void 0) {
				const err8 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "status" },
					message: "must have required property 'status'"
				};
				if (vErrors === null) vErrors = [err8];
				else vErrors.push(err8);
				errors++;
			}
			if (data.observedTick === void 0) {
				const err9 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "observedTick" },
					message: "must have required property 'observedTick'"
				};
				if (vErrors === null) vErrors = [err9];
				else vErrors.push(err9);
				errors++;
			}
			if (data.objective === void 0) {
				const err10 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "objective" },
					message: "must have required property 'objective'"
				};
				if (vErrors === null) vErrors = [err10];
				else vErrors.push(err10);
				errors++;
			}
			if (data.target === void 0) {
				const err11 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "target" },
					message: "must have required property 'target'"
				};
				if (vErrors === null) vErrors = [err11];
				else vErrors.push(err11);
				errors++;
			}
			if (data.approach === void 0) {
				const err12 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "approach" },
					message: "must have required property 'approach'"
				};
				if (vErrors === null) vErrors = [err12];
				else vErrors.push(err12);
				errors++;
			}
			if (data.phase === void 0) {
				const err13 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "phase" },
					message: "must have required property 'phase'"
				};
				if (vErrors === null) vErrors = [err13];
				else vErrors.push(err13);
				errors++;
			}
			if (data.switchReason === void 0) {
				const err14 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "switchReason" },
					message: "must have required property 'switchReason'"
				};
				if (vErrors === null) vErrors = [err14];
				else vErrors.push(err14);
				errors++;
			}
			for (const key1 in data) if (!(key1 === "seat" || key1 === "status" || key1 === "observedTick" || key1 === "objective" || key1 === "target" || key1 === "approach" || key1 === "phase" || key1 === "switchReason")) {
				const err15 = {
					instancePath,
					schemaPath: "#/oneOf/1/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key1 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err15];
				else vErrors.push(err15);
				errors++;
			}
			if (data.seat !== void 0) {
				let data2 = data.seat;
				if (typeof data2 !== "string") {
					const err16 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/seat/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err16];
					else vErrors.push(err16);
					errors++;
				}
				if (!(data2 === "one" || data2 === "two" || data2 === "three" || data2 === "four")) {
					const err17 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/seat/enum",
						keyword: "enum",
						params: { allowedValues: schema32.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err17];
					else vErrors.push(err17);
					errors++;
				}
			}
			if (data.status !== void 0) {
				if ("available" !== data.status) {
					const err18 = {
						instancePath: instancePath + "/status",
						schemaPath: "#/oneOf/1/properties/status/const",
						keyword: "const",
						params: { allowedValue: "available" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err18];
					else vErrors.push(err18);
					errors++;
				}
			}
			if (data.observedTick !== void 0) {
				let data4 = data.observedTick;
				if (!(typeof data4 == "number" && !(data4 % 1) && !isNaN(data4) && isFinite(data4))) {
					const err19 = {
						instancePath: instancePath + "/observedTick",
						schemaPath: "#/oneOf/1/properties/observedTick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err19];
					else vErrors.push(err19);
					errors++;
				}
				if (typeof data4 == "number" && isFinite(data4)) {
					if (data4 > 9007199254740991 || isNaN(data4)) {
						const err20 = {
							instancePath: instancePath + "/observedTick",
							schemaPath: "#/oneOf/1/properties/observedTick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err20];
						else vErrors.push(err20);
						errors++;
					}
					if (data4 < 0 || isNaN(data4)) {
						const err21 = {
							instancePath: instancePath + "/observedTick",
							schemaPath: "#/oneOf/1/properties/observedTick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err21];
						else vErrors.push(err21);
						errors++;
					}
				}
			}
			if (data.objective !== void 0) {
				let data5 = data.objective;
				if (typeof data5 !== "string") {
					const err22 = {
						instancePath: instancePath + "/objective",
						schemaPath: "#/oneOf/1/properties/objective/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err22];
					else vErrors.push(err22);
					errors++;
				}
				if (!(data5 === "collect" || data5 === "farm" || data5 === "attack" || data5 === "explore" || data5 === "survive" || data5 === "wait")) {
					const err23 = {
						instancePath: instancePath + "/objective",
						schemaPath: "#/oneOf/1/properties/objective/enum",
						keyword: "enum",
						params: { allowedValues: schema33.oneOf[1].properties.objective.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err23];
					else vErrors.push(err23);
					errors++;
				}
			}
			if (data.target !== void 0) {
				let data6 = data.target;
				const _errs20 = errors;
				let valid5 = false;
				const _errs21 = errors;
				if (!validate22(data6, {
					instancePath: instancePath + "/target",
					parentData: data,
					parentDataProperty: "target",
					rootData,
					dynamicAnchors
				})) {
					vErrors = vErrors === null ? validate22.errors : vErrors.concat(validate22.errors);
					errors = vErrors.length;
				}
				var _valid1 = _errs21 === errors;
				valid5 = valid5 || _valid1;
				const _errs22 = errors;
				if (data6 !== null) {
					const err24 = {
						instancePath: instancePath + "/target",
						schemaPath: "#/oneOf/1/properties/target/anyOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err24];
					else vErrors.push(err24);
					errors++;
				}
				var _valid1 = _errs22 === errors;
				valid5 = valid5 || _valid1;
				if (!valid5) {
					const err25 = {
						instancePath: instancePath + "/target",
						schemaPath: "#/oneOf/1/properties/target/anyOf",
						keyword: "anyOf",
						params: {},
						message: "must match a schema in anyOf"
					};
					if (vErrors === null) vErrors = [err25];
					else vErrors.push(err25);
					errors++;
				} else {
					errors = _errs20;
					if (vErrors !== null) {
						if (_errs20) vErrors.length = _errs20;
						else vErrors = null;
					}
				}
			}
			if (data.approach !== void 0) {
				let data7 = data.approach;
				const _errs25 = errors;
				let valid6 = false;
				const _errs26 = errors;
				if (data7 && typeof data7 == "object" && !Array.isArray(data7)) {
					if (data7.column === void 0) {
						const err26 = {
							instancePath: instancePath + "/approach",
							schemaPath: "#/$defs/cell/required",
							keyword: "required",
							params: { missingProperty: "column" },
							message: "must have required property 'column'"
						};
						if (vErrors === null) vErrors = [err26];
						else vErrors.push(err26);
						errors++;
					}
					if (data7.row === void 0) {
						const err27 = {
							instancePath: instancePath + "/approach",
							schemaPath: "#/$defs/cell/required",
							keyword: "required",
							params: { missingProperty: "row" },
							message: "must have required property 'row'"
						};
						if (vErrors === null) vErrors = [err27];
						else vErrors.push(err27);
						errors++;
					}
					for (const key2 in data7) if (!(key2 === "column" || key2 === "row")) {
						const err28 = {
							instancePath: instancePath + "/approach",
							schemaPath: "#/$defs/cell/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key2 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err28];
						else vErrors.push(err28);
						errors++;
					}
					if (data7.column !== void 0) {
						let data8 = data7.column;
						if (!(typeof data8 == "number" && !(data8 % 1) && !isNaN(data8) && isFinite(data8))) {
							const err29 = {
								instancePath: instancePath + "/approach/column",
								schemaPath: "#/$defs/cell/properties/column/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err29];
							else vErrors.push(err29);
							errors++;
						}
						if (typeof data8 == "number" && isFinite(data8)) {
							if (data8 > 14 || isNaN(data8)) {
								const err30 = {
									instancePath: instancePath + "/approach/column",
									schemaPath: "#/$defs/cell/properties/column/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 14
									},
									message: "must be <= 14"
								};
								if (vErrors === null) vErrors = [err30];
								else vErrors.push(err30);
								errors++;
							}
							if (data8 < 0 || isNaN(data8)) {
								const err31 = {
									instancePath: instancePath + "/approach/column",
									schemaPath: "#/$defs/cell/properties/column/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err31];
								else vErrors.push(err31);
								errors++;
							}
						}
					}
					if (data7.row !== void 0) {
						let data9 = data7.row;
						if (!(typeof data9 == "number" && !(data9 % 1) && !isNaN(data9) && isFinite(data9))) {
							const err32 = {
								instancePath: instancePath + "/approach/row",
								schemaPath: "#/$defs/cell/properties/row/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err32];
							else vErrors.push(err32);
							errors++;
						}
						if (typeof data9 == "number" && isFinite(data9)) {
							if (data9 > 12 || isNaN(data9)) {
								const err33 = {
									instancePath: instancePath + "/approach/row",
									schemaPath: "#/$defs/cell/properties/row/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 12
									},
									message: "must be <= 12"
								};
								if (vErrors === null) vErrors = [err33];
								else vErrors.push(err33);
								errors++;
							}
							if (data9 < 0 || isNaN(data9)) {
								const err34 = {
									instancePath: instancePath + "/approach/row",
									schemaPath: "#/$defs/cell/properties/row/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err34];
								else vErrors.push(err34);
								errors++;
							}
						}
					}
				} else {
					const err35 = {
						instancePath: instancePath + "/approach",
						schemaPath: "#/$defs/cell/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err35];
					else vErrors.push(err35);
					errors++;
				}
				var _valid2 = _errs26 === errors;
				valid6 = valid6 || _valid2;
				const _errs34 = errors;
				if (data7 !== null) {
					const err36 = {
						instancePath: instancePath + "/approach",
						schemaPath: "#/oneOf/1/properties/approach/anyOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err36];
					else vErrors.push(err36);
					errors++;
				}
				var _valid2 = _errs34 === errors;
				valid6 = valid6 || _valid2;
				if (!valid6) {
					const err37 = {
						instancePath: instancePath + "/approach",
						schemaPath: "#/oneOf/1/properties/approach/anyOf",
						keyword: "anyOf",
						params: {},
						message: "must match a schema in anyOf"
					};
					if (vErrors === null) vErrors = [err37];
					else vErrors.push(err37);
					errors++;
				} else {
					errors = _errs25;
					if (vErrors !== null) {
						if (_errs25) vErrors.length = _errs25;
						else vErrors = null;
					}
				}
			}
			if (data.phase !== void 0) {
				let data10 = data.phase;
				if (typeof data10 !== "string") {
					const err38 = {
						instancePath: instancePath + "/phase",
						schemaPath: "#/oneOf/1/properties/phase/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err38];
					else vErrors.push(err38);
					errors++;
				}
				if (!(data10 === "approach" || data10 === "plant" || data10 === "escape" || data10 === "reengage" || data10 === "evade" || data10 === "waiting")) {
					const err39 = {
						instancePath: instancePath + "/phase",
						schemaPath: "#/oneOf/1/properties/phase/enum",
						keyword: "enum",
						params: { allowedValues: schema33.oneOf[1].properties.phase.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err39];
					else vErrors.push(err39);
					errors++;
				}
			}
			if (data.switchReason !== void 0) {
				let data11 = data.switchReason;
				if (typeof data11 !== "string") {
					const err40 = {
						instancePath: instancePath + "/switchReason",
						schemaPath: "#/oneOf/1/properties/switchReason/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err40];
					else vErrors.push(err40);
					errors++;
				}
				if (!(data11 === "initial" || data11 === "completed" || data11 === "targetInvalid" || data11 === "visibilityLost" || data11 === "betterOpportunity" || data11 === "stalled" || data11 === "noTargets" || data11 === "safetyInterrupt" || data11 === "resumed")) {
					const err41 = {
						instancePath: instancePath + "/switchReason",
						schemaPath: "#/oneOf/1/properties/switchReason/enum",
						keyword: "enum",
						params: { allowedValues: schema33.oneOf[1].properties.switchReason.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err41];
					else vErrors.push(err41);
					errors++;
				}
			}
		} else {
			const err42 = {
				instancePath,
				schemaPath: "#/oneOf/1/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err42];
			else vErrors.push(err42);
			errors++;
		}
		var _valid0 = _errs8 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else if (_valid0) {
			valid0 = true;
			passing0 = 1;
			if (props0 !== true) props0 = true;
		}
		if (!valid0) {
			const err43 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err43];
			else vErrors.push(err43);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate21.errors = vErrors;
		evaluated0.props = props0;
		return errors === 0;
	}
	validate21.evaluated = {
		"dynamicProps": true,
		"dynamicItems": false
	};
	function validate20(data, { instancePath = "", parentData, parentDataProperty, rootData = data, dynamicAnchors = {} } = {}) {
		let vErrors = null;
		let errors = 0;
		const evaluated0 = validate20.evaluated;
		if (evaluated0.dynamicProps) evaluated0.props = void 0;
		if (evaluated0.dynamicItems) evaluated0.items = void 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.protocolVersion === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "protocolVersion" },
					message: "must have required property 'protocolVersion'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.roomId === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "roomId" },
					message: "must have required property 'roomId'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.roundId === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "roundId" },
					message: "must have required property 'roundId'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.tick === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "tick" },
					message: "must have required property 'tick'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.revision === void 0) {
				const err4 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "revision" },
					message: "must have required property 'revision'"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.youSeat === void 0) {
				const err5 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "youSeat" },
					message: "must have required property 'youSeat'"
				};
				if (vErrors === null) vErrors = [err5];
				else vErrors.push(err5);
				errors++;
			}
			if (data.cpus === void 0) {
				const err6 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "cpus" },
					message: "must have required property 'cpus'"
				};
				if (vErrors === null) vErrors = [err6];
				else vErrors.push(err6);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "protocolVersion" || key0 === "roomId" || key0 === "roundId" || key0 === "tick" || key0 === "revision" || key0 === "youSeat" || key0 === "cpus")) {
				const err7 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err7];
				else vErrors.push(err7);
				errors++;
			}
			if (data.protocolVersion !== void 0) {
				if (1 !== data.protocolVersion) {
					const err8 = {
						instancePath: instancePath + "/protocolVersion",
						schemaPath: "#/properties/protocolVersion/const",
						keyword: "const",
						params: { allowedValue: 1 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
			}
			if (data.roomId !== void 0) {
				let data1 = data.roomId;
				if (typeof data1 === "string") {
					if (!pattern4.test(data1)) {
						const err9 = {
							instancePath: instancePath + "/roomId",
							schemaPath: "#/properties/roomId/pattern",
							keyword: "pattern",
							params: { pattern: "^[0-9a-f]{32}$" },
							message: "must match pattern \"^[0-9a-f]{32}$\""
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
				} else {
					const err10 = {
						instancePath: instancePath + "/roomId",
						schemaPath: "#/properties/roomId/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
			if (data.roundId !== void 0) {
				let data2 = data.roundId;
				if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
					const err11 = {
						instancePath: instancePath + "/roundId",
						schemaPath: "#/properties/roundId/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err11];
					else vErrors.push(err11);
					errors++;
				}
				if (typeof data2 == "number" && isFinite(data2)) {
					if (data2 > 2147483647 || isNaN(data2)) {
						const err12 = {
							instancePath: instancePath + "/roundId",
							schemaPath: "#/properties/roundId/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err12];
						else vErrors.push(err12);
						errors++;
					}
					if (data2 < 1 || isNaN(data2)) {
						const err13 = {
							instancePath: instancePath + "/roundId",
							schemaPath: "#/properties/roundId/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err13];
						else vErrors.push(err13);
						errors++;
					}
				}
			}
			if (data.tick !== void 0) {
				let data3 = data.tick;
				if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3) && isFinite(data3))) {
					const err14 = {
						instancePath: instancePath + "/tick",
						schemaPath: "#/properties/tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err14];
					else vErrors.push(err14);
					errors++;
				}
				if (typeof data3 == "number" && isFinite(data3)) {
					if (data3 > 9007199254740991 || isNaN(data3)) {
						const err15 = {
							instancePath: instancePath + "/tick",
							schemaPath: "#/properties/tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err15];
						else vErrors.push(err15);
						errors++;
					}
					if (data3 < 0 || isNaN(data3)) {
						const err16 = {
							instancePath: instancePath + "/tick",
							schemaPath: "#/properties/tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err16];
						else vErrors.push(err16);
						errors++;
					}
				}
			}
			if (data.revision !== void 0) {
				let data4 = data.revision;
				if (!(typeof data4 == "number" && !(data4 % 1) && !isNaN(data4) && isFinite(data4))) {
					const err17 = {
						instancePath: instancePath + "/revision",
						schemaPath: "#/properties/revision/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err17];
					else vErrors.push(err17);
					errors++;
				}
				if (typeof data4 == "number" && isFinite(data4)) {
					if (data4 > 2147483647 || isNaN(data4)) {
						const err18 = {
							instancePath: instancePath + "/revision",
							schemaPath: "#/properties/revision/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err18];
						else vErrors.push(err18);
						errors++;
					}
					if (data4 < 1 || isNaN(data4)) {
						const err19 = {
							instancePath: instancePath + "/revision",
							schemaPath: "#/properties/revision/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err19];
						else vErrors.push(err19);
						errors++;
					}
				}
			}
			if (data.youSeat !== void 0) {
				let data5 = data.youSeat;
				if (typeof data5 !== "string") {
					const err20 = {
						instancePath: instancePath + "/youSeat",
						schemaPath: "#/$defs/seat/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err20];
					else vErrors.push(err20);
					errors++;
				}
				if (!(data5 === "one" || data5 === "two" || data5 === "three" || data5 === "four")) {
					const err21 = {
						instancePath: instancePath + "/youSeat",
						schemaPath: "#/$defs/seat/enum",
						keyword: "enum",
						params: { allowedValues: schema32.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err21];
					else vErrors.push(err21);
					errors++;
				}
			}
			if (data.cpus !== void 0) {
				let data6 = data.cpus;
				if (Array.isArray(data6)) {
					if (data6.length > 3) {
						const err22 = {
							instancePath: instancePath + "/cpus",
							schemaPath: "#/properties/cpus/maxItems",
							keyword: "maxItems",
							params: { limit: 3 },
							message: "must NOT have more than 3 items"
						};
						if (vErrors === null) vErrors = [err22];
						else vErrors.push(err22);
						errors++;
					}
					const len0 = data6.length;
					for (let i0 = 0; i0 < len0; i0++) if (!validate21(data6[i0], {
						instancePath: instancePath + "/cpus/" + i0,
						parentData: data6,
						parentDataProperty: i0,
						rootData,
						dynamicAnchors
					})) {
						vErrors = vErrors === null ? validate21.errors : vErrors.concat(validate21.errors);
						errors = vErrors.length;
					}
				} else {
					const err23 = {
						instancePath: instancePath + "/cpus",
						schemaPath: "#/properties/cpus/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err23];
					else vErrors.push(err23);
					errors++;
				}
			}
		} else {
			const err24 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err24];
			else vErrors.push(err24);
			errors++;
		}
		validate20.errors = vErrors;
		return errors === 0;
	}
	validate20.evaluated = {
		"props": true,
		"dynamicProps": false,
		"dynamicItems": false
	};
}));
//#endregion
export default require_cpu_debug();
