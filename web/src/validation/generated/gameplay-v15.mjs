//#region \0rolldown/runtime.js
var __commonJSMin = (cb, mod) => () => (mod || (cb((mod = { exports: {} }).exports, mod, mod.exports), cb = null), mod.exports);
//#endregion
//#region \0grenadiers-validator-helper/ajv/dist/runtime/ucs2length
var require_ucs2length = /* @__PURE__ */ __commonJSMin(((exports) => {
	Object.defineProperty(exports, "__esModule", { value: true });
	function ucs2length(str) {
		const len = str.length;
		let length = 0;
		let pos = 0;
		let value;
		while (pos < len) {
			length++;
			value = str.charCodeAt(pos++);
			if (value >= 55296 && value <= 56319 && pos < len) {
				value = str.charCodeAt(pos);
				if ((value & 64512) === 56320) pos++;
			}
		}
		return length;
	}
	exports.default = ucs2length;
	ucs2length.code = "require(\"ajv/dist/runtime/ucs2length\").default";
}));
//#endregion
//#region \0grenadiers-validator-helper/fast-deep-equal
var require_fast_deep_equal = /* @__PURE__ */ __commonJSMin(((exports, module) => {
	module.exports = function equal(a, b) {
		if (a === b) return true;
		if (a && b && typeof a == "object" && typeof b == "object") {
			if (a.constructor !== b.constructor) return false;
			var length, i, keys;
			if (Array.isArray(a)) {
				length = a.length;
				if (length != b.length) return false;
				for (i = length; i-- !== 0;) if (!equal(a[i], b[i])) return false;
				return true;
			}
			if (a.constructor === RegExp) return a.source === b.source && a.flags === b.flags;
			if (a.valueOf !== Object.prototype.valueOf) return a.valueOf() === b.valueOf();
			if (a.toString !== Object.prototype.toString) return a.toString() === b.toString();
			keys = Object.keys(a);
			length = keys.length;
			if (length !== Object.keys(b).length) return false;
			for (i = length; i-- !== 0;) if (!Object.prototype.hasOwnProperty.call(b, keys[i])) return false;
			for (i = length; i-- !== 0;) {
				var key = keys[i];
				if (!equal(a[key], b[key])) return false;
			}
			return true;
		}
		return a !== a && b !== b;
	};
}));
//#endregion
//#region \0grenadiers-validator-helper/ajv/dist/runtime/equal
var require_equal = /* @__PURE__ */ __commonJSMin(((exports) => {
	Object.defineProperty(exports, "__esModule", { value: true });
	var equal = require_fast_deep_equal();
	equal.code = "require(\"ajv/dist/runtime/equal\").default";
	exports.default = equal;
}));
//#endregion
//#region \0grenadiers-validator-entry/gameplay-v15.cjs
var require_gameplay_v15 = /* @__PURE__ */ __commonJSMin(((exports, module) => {
	module.exports = validate132;
	module.exports.default = validate132;
	function validate12(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.protocolVersion === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "protocolVersion" },
					message: "must have required property 'protocolVersion'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "protocolVersion")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("authenticated" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "authenticated" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.protocolVersion !== void 0) {
				if (15 !== data.protocolVersion) {
					const err4 = {
						instancePath: instancePath + "/protocolVersion",
						schemaPath: "#/$defs/GameplayVersion/const",
						keyword: "const",
						params: { allowedValue: 15 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
		} else {
			const err5 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err5];
			else vErrors.push(err5);
			errors++;
		}
		validate12.errors = vErrors;
		return errors === 0;
	}
	function validate14(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.supportedVersion === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "supportedVersion" },
					message: "must have required property 'supportedVersion'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "supportedVersion")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("incompatibleProtocol" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "incompatibleProtocol" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.supportedVersion !== void 0) {
				let data1 = data.supportedVersion;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err4 = {
						instancePath: instancePath + "/supportedVersion",
						schemaPath: "#/$defs/ProtocolVersionInput/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 65535 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/supportedVersion",
							schemaPath: "#/$defs/ProtocolVersionInput/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 65535
							},
							message: "must be <= 65535"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/supportedVersion",
							schemaPath: "#/$defs/ProtocolVersionInput/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
				}
			}
		} else {
			const err7 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate14.errors = vErrors;
		return errors === 0;
	}
	var schema30 = { "enum": [
		"one",
		"two",
		"three",
		"four"
	] };
	var schema19 = {
		"type": "object",
		"additionalProperties": false,
		"required": [
			"roomId",
			"roomCode",
			"roundId",
			"revision",
			"tick",
			"rulesVersion",
			"mapId",
			"roundNumber",
			"scores",
			"lastRound",
			"seats",
			"phase",
			"settings",
			"settingsRevision",
			"settingsEditor",
			"availableItems",
			"configuration"
		],
		"properties": {
			"roomId": { "$ref": "#/$defs/RoomId" },
			"roomCode": { "$ref": "#/$defs/RoomCode" },
			"roundId": { "$ref": "#/$defs/Counter" },
			"revision": { "$ref": "#/$defs/Counter" },
			"tick": { "$ref": "#/$defs/Tick" },
			"rulesVersion": { "$ref": "#/$defs/RulesVersion" },
			"mapId": { "$ref": "#/$defs/MapId" },
			"roundNumber": { "$ref": "#/$defs/RoundCount" },
			"scores": { "$ref": "#/$defs/Scores" },
			"lastRound": { "oneOf": [{ "$ref": "#/$defs/CompletedRound" }, { "type": "null" }] },
			"seats": {
				"type": "array",
				"minItems": 2,
				"maxItems": 4,
				"items": { "$ref": "#/$defs/SeatStatus" }
			},
			"phase": { "$ref": "#/$defs/RoomPhase" },
			"settings": { "$ref": "#/$defs/GameSettings" },
			"settingsRevision": { "$ref": "#/$defs/Counter" },
			"settingsEditor": {
				"description": "The seat currently granted permission to edit match settings, or null when settings are read-only.",
				"oneOf": [{ "$ref": "#/$defs/Seat" }, { "type": "null" }]
			},
			"availableItems": {
				"type": "array",
				"minItems": 0,
				"maxItems": 22,
				"uniqueItems": true,
				"items": { "$ref": "#/$defs/ItemId" }
			},
			"configuration": { "$ref": "#/$defs/RoomConfiguration" }
		}
	};
	var schema26 = { "enum": [
		"fixed_v1",
		"generated_v1",
		"generated_v2"
	] };
	var schema73 = { "enum": [
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
	] };
	var func2 = Object.prototype.hasOwnProperty;
	var func3 = require_ucs2length().default;
	var func0 = require_equal().default;
	var pattern0 = /* @__PURE__ */ new RegExp("^[0-9a-f]{32}$", "u");
	var pattern1 = /* @__PURE__ */ new RegExp("^[A-HJ-NP-Z2-9]{8}$", "u");
	function validate19(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.seat === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "seat" },
					message: "must have required property 'seat'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.wins === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "wins" },
					message: "must have required property 'wins'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "seat" || key0 === "wins")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
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
				if (!(data0 === "one" || data0 === "two" || data0 === "three" || data0 === "four")) {
					const err3 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/Seat/enum",
						keyword: "enum",
						params: { allowedValues: schema30.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.wins !== void 0) {
				let data1 = data.wins;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err4 = {
						instancePath: instancePath + "/wins",
						schemaPath: "#/$defs/Wins/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 9 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/wins",
							schemaPath: "#/$defs/Wins/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9
							},
							message: "must be <= 9"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/wins",
							schemaPath: "#/$defs/Wins/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
				}
			}
		} else {
			const err7 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate19.errors = vErrors;
		return errors === 0;
	}
	function validate18(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (Array.isArray(data)) {
			if (data.length > 4) {
				const err0 = {
					instancePath,
					schemaPath: "#/maxItems",
					keyword: "maxItems",
					params: { limit: 4 },
					message: "must NOT have more than 4 items"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.length < 2) {
				const err1 = {
					instancePath,
					schemaPath: "#/minItems",
					keyword: "minItems",
					params: { limit: 2 },
					message: "must NOT have fewer than 2 items"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			const len0 = data.length;
			for (let i0 = 0; i0 < len0; i0++) if (!validate19(data[i0], {
				instancePath: instancePath + "/" + i0,
				parentData: data,
				parentDataProperty: i0,
				rootData
			})) {
				vErrors = vErrors === null ? validate19.errors : vErrors.concat(validate19.errors);
				errors = vErrors.length;
			}
		} else {
			const err2 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "array" },
				message: "must be array"
			};
			if (vErrors === null) vErrors = [err2];
			else vErrors.push(err2);
			errors++;
		}
		validate18.errors = vErrors;
		return errors === 0;
	}
	var schema39 = {
		"type": "object",
		"additionalProperties": false,
		"required": ["type", "reason"],
		"properties": {
			"type": { "const": "draw" },
			"reason": { "enum": ["mutualElimination", "timeout"] }
		}
	};
	function validate24(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.seat === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "seat" },
					message: "must have required property 'seat'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "seat")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("winner" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "winner" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.seat !== void 0) {
				let data1 = data.seat;
				if (!(data1 === "one" || data1 === "two" || data1 === "three" || data1 === "four")) {
					const err4 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/Seat/enum",
						keyword: "enum",
						params: { allowedValues: schema30.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
		} else {
			const err5 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err5];
			else vErrors.push(err5);
			errors++;
		}
		validate24.errors = vErrors;
		return errors === 0;
	}
	function validate23(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate24(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate24.errors : vErrors.concat(validate24.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/$defs/DrawResult/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.reason === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/$defs/DrawResult/required",
					keyword: "required",
					params: { missingProperty: "reason" },
					message: "must have required property 'reason'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "reason")) {
				const err2 = {
					instancePath,
					schemaPath: "#/$defs/DrawResult/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("draw" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/$defs/DrawResult/properties/type/const",
						keyword: "const",
						params: { allowedValue: "draw" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.reason !== void 0) {
				let data1 = data.reason;
				if (!(data1 === "mutualElimination" || data1 === "timeout")) {
					const err4 = {
						instancePath: instancePath + "/reason",
						schemaPath: "#/$defs/DrawResult/properties/reason/enum",
						keyword: "enum",
						params: { allowedValues: schema39.properties.reason.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
		} else {
			const err5 = {
				instancePath,
				schemaPath: "#/$defs/DrawResult/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err5];
			else vErrors.push(err5);
			errors++;
		}
		var _valid0 = _errs2 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else if (_valid0) {
			valid0 = true;
			passing0 = 1;
		}
		if (!valid0) {
			const err6 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err6];
			else vErrors.push(err6);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate23.errors = vErrors;
		return errors === 0;
	}
	function validate22(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.roundId === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "roundId" },
					message: "must have required property 'roundId'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.roundNumber === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "roundNumber" },
					message: "must have required property 'roundNumber'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.endedAt === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "endedAt" },
					message: "must have required property 'endedAt'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.result === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "result" },
					message: "must have required property 'result'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "roundId" || key0 === "roundNumber" || key0 === "endedAt" || key0 === "result")) {
				const err4 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.roundId !== void 0) {
				let data0 = data.roundId;
				if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0) && isFinite(data0))) {
					const err5 = {
						instancePath: instancePath + "/roundId",
						schemaPath: "#/$defs/Counter/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
				if (typeof data0 == "number" && isFinite(data0)) {
					if (data0 > 2147483647 || isNaN(data0)) {
						const err6 = {
							instancePath: instancePath + "/roundId",
							schemaPath: "#/$defs/Counter/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					if (data0 < 1 || isNaN(data0)) {
						const err7 = {
							instancePath: instancePath + "/roundId",
							schemaPath: "#/$defs/Counter/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
				}
			}
			if (data.roundNumber !== void 0) {
				let data1 = data.roundNumber;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err8 = {
						instancePath: instancePath + "/roundNumber",
						schemaPath: "#/$defs/Counter/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 2147483647 || isNaN(data1)) {
						const err9 = {
							instancePath: instancePath + "/roundNumber",
							schemaPath: "#/$defs/Counter/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err10 = {
							instancePath: instancePath + "/roundNumber",
							schemaPath: "#/$defs/Counter/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err10];
						else vErrors.push(err10);
						errors++;
					}
				}
			}
			if (data.endedAt !== void 0) {
				let data2 = data.endedAt;
				if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
					const err11 = {
						instancePath: instancePath + "/endedAt",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err11];
					else vErrors.push(err11);
					errors++;
				}
				if (typeof data2 == "number" && isFinite(data2)) {
					if (data2 > 9007199254740991 || isNaN(data2)) {
						const err12 = {
							instancePath: instancePath + "/endedAt",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err12];
						else vErrors.push(err12);
						errors++;
					}
					if (data2 < 0 || isNaN(data2)) {
						const err13 = {
							instancePath: instancePath + "/endedAt",
							schemaPath: "#/$defs/Tick/minimum",
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
			if (data.result !== void 0) {
				if (!validate23(data.result, {
					instancePath: instancePath + "/result",
					parentData: data,
					parentDataProperty: "result",
					rootData
				})) {
					vErrors = vErrors === null ? validate23.errors : vErrors.concat(validate23.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err14 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err14];
			else vErrors.push(err14);
			errors++;
		}
		validate22.errors = vErrors;
		return errors === 0;
	}
	var schema45 = { "enum": ["easy", "standard"] };
	function validate30(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.difficulty === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "difficulty" },
					message: "must have required property 'difficulty'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "difficulty")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("cpu" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "cpu" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.difficulty !== void 0) {
				let data1 = data.difficulty;
				if (!(data1 === "easy" || data1 === "standard")) {
					const err4 = {
						instancePath: instancePath + "/difficulty",
						schemaPath: "#/$defs/Difficulty/enum",
						keyword: "enum",
						params: { allowedValues: schema45.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
		} else {
			const err5 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err5];
			else vErrors.push(err5);
			errors++;
		}
		validate30.errors = vErrors;
		return errors === 0;
	}
	function validate29(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/$defs/HumanController/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/$defs/HumanController/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.type !== void 0) {
				if ("human" !== data.type) {
					const err2 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/$defs/HumanController/properties/type/const",
						keyword: "const",
						params: { allowedValue: "human" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err2];
					else vErrors.push(err2);
					errors++;
				}
			}
		} else {
			const err3 = {
				instancePath,
				schemaPath: "#/$defs/HumanController/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs6 = errors;
		if (!validate30(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate30.errors : vErrors.concat(validate30.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs6 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else if (_valid0) {
			valid0 = true;
			passing0 = 1;
		}
		if (!valid0) {
			const err4 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err4];
			else vErrors.push(err4);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate29.errors = vErrors;
		return errors === 0;
	}
	function validate34(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.expiresAt === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "expiresAt" },
					message: "must have required property 'expiresAt'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "expiresAt")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("disconnected" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "disconnected" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.expiresAt !== void 0) {
				let data1 = data.expiresAt;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err4 = {
						instancePath: instancePath + "/expiresAt",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 9007199254740991 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/expiresAt",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/expiresAt",
							schemaPath: "#/$defs/Tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
				}
			}
		} else {
			const err7 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate34.errors = vErrors;
		return errors === 0;
	}
	function validate33(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/$defs/VacantConnection/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/$defs/VacantConnection/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.type !== void 0) {
				if ("vacant" !== data.type) {
					const err2 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/$defs/VacantConnection/properties/type/const",
						keyword: "const",
						params: { allowedValue: "vacant" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err2];
					else vErrors.push(err2);
					errors++;
				}
			}
		} else {
			const err3 = {
				instancePath,
				schemaPath: "#/$defs/VacantConnection/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs6 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err4 = {
					instancePath,
					schemaPath: "#/$defs/ConnectedConnection/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			for (const key1 in data) if (!(key1 === "type")) {
				const err5 = {
					instancePath,
					schemaPath: "#/$defs/ConnectedConnection/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key1 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err5];
				else vErrors.push(err5);
				errors++;
			}
			if (data.type !== void 0) {
				if ("connected" !== data.type) {
					const err6 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/$defs/ConnectedConnection/properties/type/const",
						keyword: "const",
						params: { allowedValue: "connected" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
			}
		} else {
			const err7 = {
				instancePath,
				schemaPath: "#/$defs/ConnectedConnection/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		var _valid0 = _errs6 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else {
			if (_valid0) {
				valid0 = true;
				passing0 = 1;
			}
			const _errs11 = errors;
			if (!validate34(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate34.errors : vErrors.concat(validate34.errors);
				errors = vErrors.length;
			}
			var _valid0 = _errs11 === errors;
			if (_valid0 && valid0) {
				valid0 = false;
				passing0 = [passing0, 2];
			} else {
				if (_valid0) {
					valid0 = true;
					passing0 = 2;
				}
				const _errs12 = errors;
				if (data && typeof data == "object" && !Array.isArray(data)) {
					if (data.type === void 0) {
						const err8 = {
							instancePath,
							schemaPath: "#/$defs/ExpiredConnection/required",
							keyword: "required",
							params: { missingProperty: "type" },
							message: "must have required property 'type'"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
					for (const key2 in data) if (!(key2 === "type")) {
						const err9 = {
							instancePath,
							schemaPath: "#/$defs/ExpiredConnection/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key2 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
					if (data.type !== void 0) {
						if ("expired" !== data.type) {
							const err10 = {
								instancePath: instancePath + "/type",
								schemaPath: "#/$defs/ExpiredConnection/properties/type/const",
								keyword: "const",
								params: { allowedValue: "expired" },
								message: "must be equal to constant"
							};
							if (vErrors === null) vErrors = [err10];
							else vErrors.push(err10);
							errors++;
						}
					}
				} else {
					const err11 = {
						instancePath,
						schemaPath: "#/$defs/ExpiredConnection/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err11];
					else vErrors.push(err11);
					errors++;
				}
				var _valid0 = _errs12 === errors;
				if (_valid0 && valid0) {
					valid0 = false;
					passing0 = [passing0, 3];
				} else {
					if (_valid0) {
						valid0 = true;
						passing0 = 3;
					}
					const _errs17 = errors;
					if (data && typeof data == "object" && !Array.isArray(data)) {
						if (data.type === void 0) {
							const err12 = {
								instancePath,
								schemaPath: "#/$defs/ServerControlledConnection/required",
								keyword: "required",
								params: { missingProperty: "type" },
								message: "must have required property 'type'"
							};
							if (vErrors === null) vErrors = [err12];
							else vErrors.push(err12);
							errors++;
						}
						for (const key3 in data) if (!(key3 === "type")) {
							const err13 = {
								instancePath,
								schemaPath: "#/$defs/ServerControlledConnection/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: key3 },
								message: "must NOT have additional properties"
							};
							if (vErrors === null) vErrors = [err13];
							else vErrors.push(err13);
							errors++;
						}
						if (data.type !== void 0) {
							if ("serverControlled" !== data.type) {
								const err14 = {
									instancePath: instancePath + "/type",
									schemaPath: "#/$defs/ServerControlledConnection/properties/type/const",
									keyword: "const",
									params: { allowedValue: "serverControlled" },
									message: "must be equal to constant"
								};
								if (vErrors === null) vErrors = [err14];
								else vErrors.push(err14);
								errors++;
							}
						}
					} else {
						const err15 = {
							instancePath,
							schemaPath: "#/$defs/ServerControlledConnection/type",
							keyword: "type",
							params: { type: "object" },
							message: "must be object"
						};
						if (vErrors === null) vErrors = [err15];
						else vErrors.push(err15);
						errors++;
					}
					var _valid0 = _errs17 === errors;
					if (_valid0 && valid0) {
						valid0 = false;
						passing0 = [passing0, 4];
					} else if (_valid0) {
						valid0 = true;
						passing0 = 4;
					}
				}
			}
		}
		if (!valid0) {
			const err16 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err16];
			else vErrors.push(err16);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate33.errors = vErrors;
		return errors === 0;
	}
	function validate28(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.seat === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "seat" },
					message: "must have required property 'seat'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.occupied === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "occupied" },
					message: "must have required property 'occupied'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.ready === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "ready" },
					message: "must have required property 'ready'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.rematchReady === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "rematchReady" },
					message: "must have required property 'rematchReady'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.controller === void 0) {
				const err4 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "controller" },
					message: "must have required property 'controller'"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.connection === void 0) {
				const err5 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "connection" },
					message: "must have required property 'connection'"
				};
				if (vErrors === null) vErrors = [err5];
				else vErrors.push(err5);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "seat" || key0 === "occupied" || key0 === "ready" || key0 === "rematchReady" || key0 === "controller" || key0 === "connection")) {
				const err6 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err6];
				else vErrors.push(err6);
				errors++;
			}
			if (data.seat !== void 0) {
				let data0 = data.seat;
				if (!(data0 === "one" || data0 === "two" || data0 === "three" || data0 === "four")) {
					const err7 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/Seat/enum",
						keyword: "enum",
						params: { allowedValues: schema30.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err7];
					else vErrors.push(err7);
					errors++;
				}
			}
			if (data.occupied !== void 0) {
				if (typeof data.occupied !== "boolean") {
					const err8 = {
						instancePath: instancePath + "/occupied",
						schemaPath: "#/properties/occupied/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
			}
			if (data.ready !== void 0) {
				if (typeof data.ready !== "boolean") {
					const err9 = {
						instancePath: instancePath + "/ready",
						schemaPath: "#/properties/ready/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
			}
			if (data.rematchReady !== void 0) {
				if (typeof data.rematchReady !== "boolean") {
					const err10 = {
						instancePath: instancePath + "/rematchReady",
						schemaPath: "#/properties/rematchReady/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
			if (data.controller !== void 0) {
				if (!validate29(data.controller, {
					instancePath: instancePath + "/controller",
					parentData: data,
					parentDataProperty: "controller",
					rootData
				})) {
					vErrors = vErrors === null ? validate29.errors : vErrors.concat(validate29.errors);
					errors = vErrors.length;
				}
			}
			if (data.connection !== void 0) {
				if (!validate33(data.connection, {
					instancePath: instancePath + "/connection",
					parentData: data,
					parentDataProperty: "connection",
					rootData
				})) {
					vErrors = vErrors === null ? validate33.errors : vErrors.concat(validate33.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err11 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err11];
			else vErrors.push(err11);
			errors++;
		}
		validate28.errors = vErrors;
		return errors === 0;
	}
	var schema55 = {
		"type": "object",
		"additionalProperties": false,
		"required": [
			"type",
			"kind",
			"endsAt"
		],
		"properties": {
			"type": { "const": "countdown" },
			"kind": { "enum": ["initial", "interRound"] },
			"endsAt": { "$ref": "#/$defs/Tick" }
		}
	};
	function validate39(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.kind === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "kind" },
					message: "must have required property 'kind'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.endsAt === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "endsAt" },
					message: "must have required property 'endsAt'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "kind" || key0 === "endsAt")) {
				const err3 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.type !== void 0) {
				if ("countdown" !== data.type) {
					const err4 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "countdown" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.kind !== void 0) {
				let data1 = data.kind;
				if (!(data1 === "initial" || data1 === "interRound")) {
					const err5 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/properties/kind/enum",
						keyword: "enum",
						params: { allowedValues: schema55.properties.kind.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.endsAt !== void 0) {
				let data2 = data.endsAt;
				if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
					const err6 = {
						instancePath: instancePath + "/endsAt",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
				if (typeof data2 == "number" && isFinite(data2)) {
					if (data2 > 9007199254740991 || isNaN(data2)) {
						const err7 = {
							instancePath: instancePath + "/endsAt",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data2 < 0 || isNaN(data2)) {
						const err8 = {
							instancePath: instancePath + "/endsAt",
							schemaPath: "#/$defs/Tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
				}
			}
		} else {
			const err9 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err9];
			else vErrors.push(err9);
			errors++;
		}
		validate39.errors = vErrors;
		return errors === 0;
	}
	function validate41(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.holdUntil === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "holdUntil" },
					message: "must have required property 'holdUntil'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "holdUntil")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("result" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "result" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.holdUntil !== void 0) {
				let data1 = data.holdUntil;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err4 = {
						instancePath: instancePath + "/holdUntil",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 9007199254740991 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/holdUntil",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/holdUntil",
							schemaPath: "#/$defs/Tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
				}
			}
		} else {
			const err7 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate41.errors = vErrors;
		return errors === 0;
	}
	var schema63 = {
		"type": "object",
		"additionalProperties": false,
		"required": [
			"type",
			"seat",
			"reason"
		],
		"properties": {
			"type": { "const": "winner" },
			"seat": { "$ref": "#/$defs/Seat" },
			"reason": { "enum": ["roundWins", "forfeit"] }
		}
	};
	function validate45(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.seat === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "seat" },
					message: "must have required property 'seat'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.reason === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "reason" },
					message: "must have required property 'reason'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "seat" || key0 === "reason")) {
				const err3 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.type !== void 0) {
				if ("winner" !== data.type) {
					const err4 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "winner" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.seat !== void 0) {
				let data1 = data.seat;
				if (!(data1 === "one" || data1 === "two" || data1 === "three" || data1 === "four")) {
					const err5 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/Seat/enum",
						keyword: "enum",
						params: { allowedValues: schema30.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.reason !== void 0) {
				let data2 = data.reason;
				if (!(data2 === "roundWins" || data2 === "forfeit")) {
					const err6 = {
						instancePath: instancePath + "/reason",
						schemaPath: "#/properties/reason/enum",
						keyword: "enum",
						params: { allowedValues: schema63.properties.reason.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
			}
		} else {
			const err7 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate45.errors = vErrors;
		return errors === 0;
	}
	var schema67 = { "enum": [
		"connectionLost",
		"serverOverloaded",
		"serverStopped",
		"internalError"
	] };
	function validate47(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.reason === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "reason" },
					message: "must have required property 'reason'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "reason")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("interrupted" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "interrupted" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.reason !== void 0) {
				let data1 = data.reason;
				if (!(data1 === "connectionLost" || data1 === "serverOverloaded" || data1 === "serverStopped" || data1 === "internalError")) {
					const err4 = {
						instancePath: instancePath + "/reason",
						schemaPath: "#/$defs/MatchInterruptReason/enum",
						keyword: "enum",
						params: { allowedValues: schema67.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
		} else {
			const err5 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err5];
			else vErrors.push(err5);
			errors++;
		}
		validate47.errors = vErrors;
		return errors === 0;
	}
	function validate44(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate45(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate45.errors : vErrors.concat(validate45.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/$defs/AbandonedMatchResult/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/$defs/AbandonedMatchResult/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.type !== void 0) {
				if ("abandoned" !== data.type) {
					const err2 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/$defs/AbandonedMatchResult/properties/type/const",
						keyword: "const",
						params: { allowedValue: "abandoned" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err2];
					else vErrors.push(err2);
					errors++;
				}
			}
		} else {
			const err3 = {
				instancePath,
				schemaPath: "#/$defs/AbandonedMatchResult/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		var _valid0 = _errs2 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else {
			if (_valid0) {
				valid0 = true;
				passing0 = 1;
			}
			const _errs7 = errors;
			if (!validate47(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate47.errors : vErrors.concat(validate47.errors);
				errors = vErrors.length;
			}
			var _valid0 = _errs7 === errors;
			if (_valid0 && valid0) {
				valid0 = false;
				passing0 = [passing0, 2];
			} else if (_valid0) {
				valid0 = true;
				passing0 = 2;
			}
		}
		if (!valid0) {
			const err4 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err4];
			else vErrors.push(err4);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate44.errors = vErrors;
		return errors === 0;
	}
	function validate43(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.endedAt === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "endedAt" },
					message: "must have required property 'endedAt'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.result === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "result" },
					message: "must have required property 'result'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.rematchAt === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "rematchAt" },
					message: "must have required property 'rematchAt'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "endedAt" || key0 === "result" || key0 === "rematchAt")) {
				const err4 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.type !== void 0) {
				if ("matchResult" !== data.type) {
					const err5 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "matchResult" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.endedAt !== void 0) {
				let data1 = data.endedAt;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err6 = {
						instancePath: instancePath + "/endedAt",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 9007199254740991 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/endedAt",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err8 = {
							instancePath: instancePath + "/endedAt",
							schemaPath: "#/$defs/Tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
				}
			}
			if (data.result !== void 0) {
				if (!validate44(data.result, {
					instancePath: instancePath + "/result",
					parentData: data,
					parentDataProperty: "result",
					rootData
				})) {
					vErrors = vErrors === null ? validate44.errors : vErrors.concat(validate44.errors);
					errors = vErrors.length;
				}
			}
			if (data.rematchAt !== void 0) {
				let data3 = data.rematchAt;
				if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3) && isFinite(data3))) {
					const err9 = {
						instancePath: instancePath + "/rematchAt",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
				if (typeof data3 == "number" && isFinite(data3)) {
					if (data3 > 9007199254740991 || isNaN(data3)) {
						const err10 = {
							instancePath: instancePath + "/rematchAt",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err10];
						else vErrors.push(err10);
						errors++;
					}
					if (data3 < 0 || isNaN(data3)) {
						const err11 = {
							instancePath: instancePath + "/rematchAt",
							schemaPath: "#/$defs/Tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
				}
			}
		} else {
			const err12 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err12];
			else vErrors.push(err12);
			errors++;
		}
		validate43.errors = vErrors;
		return errors === 0;
	}
	function validate51(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.endedAt === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "endedAt" },
					message: "must have required property 'endedAt'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.reason === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "reason" },
					message: "must have required property 'reason'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "endedAt" || key0 === "reason")) {
				const err3 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.type !== void 0) {
				if ("interrupted" !== data.type) {
					const err4 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "interrupted" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.endedAt !== void 0) {
				let data1 = data.endedAt;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err5 = {
						instancePath: instancePath + "/endedAt",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 9007199254740991 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/endedAt",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/endedAt",
							schemaPath: "#/$defs/Tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
				}
			}
			if (data.reason !== void 0) {
				let data2 = data.reason;
				if (!(data2 === "connectionLost" || data2 === "serverOverloaded" || data2 === "serverStopped" || data2 === "internalError")) {
					const err8 = {
						instancePath: instancePath + "/reason",
						schemaPath: "#/$defs/MatchInterruptReason/enum",
						keyword: "enum",
						params: { allowedValues: schema67.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
			}
		} else {
			const err9 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err9];
			else vErrors.push(err9);
			errors++;
		}
		validate51.errors = vErrors;
		return errors === 0;
	}
	function validate38(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/$defs/WaitingPhase/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/$defs/WaitingPhase/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.type !== void 0) {
				if ("waiting" !== data.type) {
					const err2 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/$defs/WaitingPhase/properties/type/const",
						keyword: "const",
						params: { allowedValue: "waiting" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err2];
					else vErrors.push(err2);
					errors++;
				}
			}
		} else {
			const err3 = {
				instancePath,
				schemaPath: "#/$defs/WaitingPhase/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs6 = errors;
		if (!validate39(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate39.errors : vErrors.concat(validate39.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs6 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else {
			if (_valid0) {
				valid0 = true;
				passing0 = 1;
			}
			const _errs7 = errors;
			if (data && typeof data == "object" && !Array.isArray(data)) {
				if (data.type === void 0) {
					const err4 = {
						instancePath,
						schemaPath: "#/$defs/RunningPhase/required",
						keyword: "required",
						params: { missingProperty: "type" },
						message: "must have required property 'type'"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				for (const key1 in data) if (!(key1 === "type")) {
					const err5 = {
						instancePath,
						schemaPath: "#/$defs/RunningPhase/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key1 },
						message: "must NOT have additional properties"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
				if (data.type !== void 0) {
					if ("running" !== data.type) {
						const err6 = {
							instancePath: instancePath + "/type",
							schemaPath: "#/$defs/RunningPhase/properties/type/const",
							keyword: "const",
							params: { allowedValue: "running" },
							message: "must be equal to constant"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
				}
			} else {
				const err7 = {
					instancePath,
					schemaPath: "#/$defs/RunningPhase/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err7];
				else vErrors.push(err7);
				errors++;
			}
			var _valid0 = _errs7 === errors;
			if (_valid0 && valid0) {
				valid0 = false;
				passing0 = [passing0, 2];
			} else {
				if (_valid0) {
					valid0 = true;
					passing0 = 2;
				}
				const _errs12 = errors;
				if (!validate41(data, {
					instancePath,
					parentData,
					parentDataProperty,
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs12 === errors;
				if (_valid0 && valid0) {
					valid0 = false;
					passing0 = [passing0, 3];
				} else {
					if (_valid0) {
						valid0 = true;
						passing0 = 3;
					}
					const _errs13 = errors;
					if (!validate43(data, {
						instancePath,
						parentData,
						parentDataProperty,
						rootData
					})) {
						vErrors = vErrors === null ? validate43.errors : vErrors.concat(validate43.errors);
						errors = vErrors.length;
					}
					var _valid0 = _errs13 === errors;
					if (_valid0 && valid0) {
						valid0 = false;
						passing0 = [passing0, 4];
					} else {
						if (_valid0) {
							valid0 = true;
							passing0 = 4;
						}
						const _errs14 = errors;
						if (!validate51(data, {
							instancePath,
							parentData,
							parentDataProperty,
							rootData
						})) {
							vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
							errors = vErrors.length;
						}
						var _valid0 = _errs14 === errors;
						if (_valid0 && valid0) {
							valid0 = false;
							passing0 = [passing0, 5];
						} else if (_valid0) {
							valid0 = true;
							passing0 = 5;
						}
					}
				}
			}
		}
		if (!valid0) {
			const err8 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err8];
			else vErrors.push(err8);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate38.errors = vErrors;
		return errors === 0;
	}
	var schema72 = {
		"description": "Validated live room settings, including the enabled subset of the 22-item catalog.",
		"type": "object",
		"additionalProperties": false,
		"required": [
			"startingSpeedLevel",
			"turnAssistUnits",
			"winsToFinish",
			"bombFuseTicks",
			"enabledItems",
			"environment"
		],
		"properties": {
			"startingSpeedLevel": {
				"type": "integer",
				"minimum": 0,
				"maximum": 4
			},
			"turnAssistUnits": { "enum": [
				64,
				128,
				192,
				256
			] },
			"winsToFinish": {
				"type": "integer",
				"minimum": 1,
				"maximum": 9
			},
			"bombFuseTicks": { "enum": [
				60,
				90,
				120,
				150,
				180,
				210,
				240,
				270,
				300
			] },
			"enabledItems": {
				"type": "array",
				"minItems": 0,
				"maxItems": 22,
				"uniqueItems": true,
				"items": { "$ref": "#/$defs/ItemId" }
			},
			"environment": { "$ref": "#/$defs/Environment" }
		}
	};
	function validate54(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.startingSpeedLevel === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "startingSpeedLevel" },
					message: "must have required property 'startingSpeedLevel'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.turnAssistUnits === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "turnAssistUnits" },
					message: "must have required property 'turnAssistUnits'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.winsToFinish === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "winsToFinish" },
					message: "must have required property 'winsToFinish'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.bombFuseTicks === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "bombFuseTicks" },
					message: "must have required property 'bombFuseTicks'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.enabledItems === void 0) {
				const err4 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "enabledItems" },
					message: "must have required property 'enabledItems'"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.environment === void 0) {
				const err5 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "environment" },
					message: "must have required property 'environment'"
				};
				if (vErrors === null) vErrors = [err5];
				else vErrors.push(err5);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "startingSpeedLevel" || key0 === "turnAssistUnits" || key0 === "winsToFinish" || key0 === "bombFuseTicks" || key0 === "enabledItems" || key0 === "environment")) {
				const err6 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err6];
				else vErrors.push(err6);
				errors++;
			}
			if (data.startingSpeedLevel !== void 0) {
				let data0 = data.startingSpeedLevel;
				if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0) && isFinite(data0))) {
					const err7 = {
						instancePath: instancePath + "/startingSpeedLevel",
						schemaPath: "#/properties/startingSpeedLevel/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err7];
					else vErrors.push(err7);
					errors++;
				}
				if (typeof data0 == "number" && isFinite(data0)) {
					if (data0 > 4 || isNaN(data0)) {
						const err8 = {
							instancePath: instancePath + "/startingSpeedLevel",
							schemaPath: "#/properties/startingSpeedLevel/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 4
							},
							message: "must be <= 4"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
					if (data0 < 0 || isNaN(data0)) {
						const err9 = {
							instancePath: instancePath + "/startingSpeedLevel",
							schemaPath: "#/properties/startingSpeedLevel/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
				}
			}
			if (data.turnAssistUnits !== void 0) {
				let data1 = data.turnAssistUnits;
				if (!(data1 === 64 || data1 === 128 || data1 === 192 || data1 === 256)) {
					const err10 = {
						instancePath: instancePath + "/turnAssistUnits",
						schemaPath: "#/properties/turnAssistUnits/enum",
						keyword: "enum",
						params: { allowedValues: schema72.properties.turnAssistUnits.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
			if (data.winsToFinish !== void 0) {
				let data2 = data.winsToFinish;
				if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
					const err11 = {
						instancePath: instancePath + "/winsToFinish",
						schemaPath: "#/properties/winsToFinish/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err11];
					else vErrors.push(err11);
					errors++;
				}
				if (typeof data2 == "number" && isFinite(data2)) {
					if (data2 > 9 || isNaN(data2)) {
						const err12 = {
							instancePath: instancePath + "/winsToFinish",
							schemaPath: "#/properties/winsToFinish/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9
							},
							message: "must be <= 9"
						};
						if (vErrors === null) vErrors = [err12];
						else vErrors.push(err12);
						errors++;
					}
					if (data2 < 1 || isNaN(data2)) {
						const err13 = {
							instancePath: instancePath + "/winsToFinish",
							schemaPath: "#/properties/winsToFinish/minimum",
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
			if (data.bombFuseTicks !== void 0) {
				let data3 = data.bombFuseTicks;
				if (!(data3 === 60 || data3 === 90 || data3 === 120 || data3 === 150 || data3 === 180 || data3 === 210 || data3 === 240 || data3 === 270 || data3 === 300)) {
					const err14 = {
						instancePath: instancePath + "/bombFuseTicks",
						schemaPath: "#/properties/bombFuseTicks/enum",
						keyword: "enum",
						params: { allowedValues: schema72.properties.bombFuseTicks.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err14];
					else vErrors.push(err14);
					errors++;
				}
			}
			if (data.enabledItems !== void 0) {
				let data4 = data.enabledItems;
				if (Array.isArray(data4)) {
					if (data4.length > 22) {
						const err15 = {
							instancePath: instancePath + "/enabledItems",
							schemaPath: "#/properties/enabledItems/maxItems",
							keyword: "maxItems",
							params: { limit: 22 },
							message: "must NOT have more than 22 items"
						};
						if (vErrors === null) vErrors = [err15];
						else vErrors.push(err15);
						errors++;
					}
					if (data4.length < 0) {
						const err16 = {
							instancePath: instancePath + "/enabledItems",
							schemaPath: "#/properties/enabledItems/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err16];
						else vErrors.push(err16);
						errors++;
					}
					const len0 = data4.length;
					for (let i0 = 0; i0 < len0; i0++) {
						let data5 = data4[i0];
						if (!(data5 === "kick" || data5 === "punch" || data5 === "bracelet" || data5 === "wallPass" || data5 === "heart" || data5 === "remoteBomb" || data5 === "piercingBomb" || data5 === "powerBomb" || data5 === "landmine" || data5 === "shield" || data5 === "stopwatch" || data5 === "invisibilityCloak" || data5 === "goldenFlame" || data5 === "shortFuse" || data5 === "diarrhea" || data5 === "slowFoot" || data5 === "shortFlame" || data5 === "longFuse" || data5 === "reverseControls" || data5 === "invisibleBombs" || data5 === "oilBarrel" || data5 === "teleporter")) {
							const err17 = {
								instancePath: instancePath + "/enabledItems/" + i0,
								schemaPath: "#/$defs/ItemId/enum",
								keyword: "enum",
								params: { allowedValues: schema73.enum },
								message: "must be equal to one of the allowed values"
							};
							if (vErrors === null) vErrors = [err17];
							else vErrors.push(err17);
							errors++;
						}
					}
					let i1 = data4.length;
					let j0;
					if (i1 > 1) {
						outer0: for (; i1--;) for (j0 = i1; j0--;) if (func0(data4[i1], data4[j0])) {
							const err18 = {
								instancePath: instancePath + "/enabledItems",
								schemaPath: "#/properties/enabledItems/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i1,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i1 + " are identical)"
							};
							if (vErrors === null) vErrors = [err18];
							else vErrors.push(err18);
							errors++;
							break outer0;
						}
					}
				} else {
					const err19 = {
						instancePath: instancePath + "/enabledItems",
						schemaPath: "#/properties/enabledItems/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err19];
					else vErrors.push(err19);
					errors++;
				}
			}
			if (data.environment !== void 0) {
				let data6 = data.environment;
				if (data6 && typeof data6 == "object" && !Array.isArray(data6)) {
					if (data6.stormCloud === void 0) {
						const err20 = {
							instancePath: instancePath + "/environment",
							schemaPath: "#/$defs/Environment/required",
							keyword: "required",
							params: { missingProperty: "stormCloud" },
							message: "must have required property 'stormCloud'"
						};
						if (vErrors === null) vErrors = [err20];
						else vErrors.push(err20);
						errors++;
					}
					for (const key1 in data6) if (!(key1 === "stormCloud")) {
						const err21 = {
							instancePath: instancePath + "/environment",
							schemaPath: "#/$defs/Environment/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err21];
						else vErrors.push(err21);
						errors++;
					}
					if (data6.stormCloud !== void 0) {
						if (typeof data6.stormCloud !== "boolean") {
							const err22 = {
								instancePath: instancePath + "/environment/stormCloud",
								schemaPath: "#/$defs/Environment/properties/stormCloud/type",
								keyword: "type",
								params: { type: "boolean" },
								message: "must be boolean"
							};
							if (vErrors === null) vErrors = [err22];
							else vErrors.push(err22);
							errors++;
						}
					}
				} else {
					const err23 = {
						instancePath: instancePath + "/environment",
						schemaPath: "#/$defs/Environment/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
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
		validate54.errors = vErrors;
		return errors === 0;
	}
	var schema80 = { "enum": [
		["human", "human"],
		[
			"human",
			"human",
			"human"
		],
		[
			"human",
			"human",
			"human",
			"human"
		]
	] };
	function validate58(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.width === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/$defs/Board11x9/required",
					keyword: "required",
					params: { missingProperty: "width" },
					message: "must have required property 'width'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.height === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/$defs/Board11x9/required",
					keyword: "required",
					params: { missingProperty: "height" },
					message: "must have required property 'height'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "width" || key0 === "height")) {
				const err2 = {
					instancePath,
					schemaPath: "#/$defs/Board11x9/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.width !== void 0) {
				if (11 !== data.width) {
					const err3 = {
						instancePath: instancePath + "/width",
						schemaPath: "#/$defs/Board11x9/properties/width/const",
						keyword: "const",
						params: { allowedValue: 11 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.height !== void 0) {
				if (9 !== data.height) {
					const err4 = {
						instancePath: instancePath + "/height",
						schemaPath: "#/$defs/Board11x9/properties/height/const",
						keyword: "const",
						params: { allowedValue: 9 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
		} else {
			const err5 = {
				instancePath,
				schemaPath: "#/$defs/Board11x9/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err5];
			else vErrors.push(err5);
			errors++;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs7 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.width === void 0) {
				const err6 = {
					instancePath,
					schemaPath: "#/$defs/Board13x11/required",
					keyword: "required",
					params: { missingProperty: "width" },
					message: "must have required property 'width'"
				};
				if (vErrors === null) vErrors = [err6];
				else vErrors.push(err6);
				errors++;
			}
			if (data.height === void 0) {
				const err7 = {
					instancePath,
					schemaPath: "#/$defs/Board13x11/required",
					keyword: "required",
					params: { missingProperty: "height" },
					message: "must have required property 'height'"
				};
				if (vErrors === null) vErrors = [err7];
				else vErrors.push(err7);
				errors++;
			}
			for (const key1 in data) if (!(key1 === "width" || key1 === "height")) {
				const err8 = {
					instancePath,
					schemaPath: "#/$defs/Board13x11/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key1 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err8];
				else vErrors.push(err8);
				errors++;
			}
			if (data.width !== void 0) {
				if (13 !== data.width) {
					const err9 = {
						instancePath: instancePath + "/width",
						schemaPath: "#/$defs/Board13x11/properties/width/const",
						keyword: "const",
						params: { allowedValue: 13 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
			}
			if (data.height !== void 0) {
				if (11 !== data.height) {
					const err10 = {
						instancePath: instancePath + "/height",
						schemaPath: "#/$defs/Board13x11/properties/height/const",
						keyword: "const",
						params: { allowedValue: 11 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
		} else {
			const err11 = {
				instancePath,
				schemaPath: "#/$defs/Board13x11/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err11];
			else vErrors.push(err11);
			errors++;
		}
		var _valid0 = _errs7 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else {
			if (_valid0) {
				valid0 = true;
				passing0 = 1;
			}
			const _errs13 = errors;
			if (data && typeof data == "object" && !Array.isArray(data)) {
				if (data.width === void 0) {
					const err12 = {
						instancePath,
						schemaPath: "#/$defs/Board15x13/required",
						keyword: "required",
						params: { missingProperty: "width" },
						message: "must have required property 'width'"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				}
				if (data.height === void 0) {
					const err13 = {
						instancePath,
						schemaPath: "#/$defs/Board15x13/required",
						keyword: "required",
						params: { missingProperty: "height" },
						message: "must have required property 'height'"
					};
					if (vErrors === null) vErrors = [err13];
					else vErrors.push(err13);
					errors++;
				}
				for (const key2 in data) if (!(key2 === "width" || key2 === "height")) {
					const err14 = {
						instancePath,
						schemaPath: "#/$defs/Board15x13/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key2 },
						message: "must NOT have additional properties"
					};
					if (vErrors === null) vErrors = [err14];
					else vErrors.push(err14);
					errors++;
				}
				if (data.width !== void 0) {
					if (15 !== data.width) {
						const err15 = {
							instancePath: instancePath + "/width",
							schemaPath: "#/$defs/Board15x13/properties/width/const",
							keyword: "const",
							params: { allowedValue: 15 },
							message: "must be equal to constant"
						};
						if (vErrors === null) vErrors = [err15];
						else vErrors.push(err15);
						errors++;
					}
				}
				if (data.height !== void 0) {
					if (13 !== data.height) {
						const err16 = {
							instancePath: instancePath + "/height",
							schemaPath: "#/$defs/Board15x13/properties/height/const",
							keyword: "const",
							params: { allowedValue: 13 },
							message: "must be equal to constant"
						};
						if (vErrors === null) vErrors = [err16];
						else vErrors.push(err16);
						errors++;
					}
				}
			} else {
				const err17 = {
					instancePath,
					schemaPath: "#/$defs/Board15x13/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err17];
				else vErrors.push(err17);
				errors++;
			}
			var _valid0 = _errs13 === errors;
			if (_valid0 && valid0) {
				valid0 = false;
				passing0 = [passing0, 2];
			} else if (_valid0) {
				valid0 = true;
				passing0 = 2;
			}
		}
		if (!valid0) {
			const err18 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err18];
			else vErrors.push(err18);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate58.errors = vErrors;
		return errors === 0;
	}
	function validate57(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.controllers === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "controllers" },
					message: "must have required property 'controllers'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.cpuDifficulty === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "cpuDifficulty" },
					message: "must have required property 'cpuDifficulty'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.board === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "board" },
					message: "must have required property 'board'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.environment === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "environment" },
					message: "must have required property 'environment'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "controllers" || key0 === "cpuDifficulty" || key0 === "board" || key0 === "environment")) {
				const err4 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.controllers !== void 0) {
				let data0 = data.controllers;
				const vSchema0 = schema80.enum;
				if (!(func0(data0, vSchema0[0]) || func0(data0, vSchema0[1]) || func0(data0, vSchema0[2]))) {
					const err5 = {
						instancePath: instancePath + "/controllers",
						schemaPath: "#/$defs/HumanControllers/enum",
						keyword: "enum",
						params: { allowedValues: schema80.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.cpuDifficulty !== void 0) {
				if (data.cpuDifficulty !== null) {
					const err6 = {
						instancePath: instancePath + "/cpuDifficulty",
						schemaPath: "#/properties/cpuDifficulty/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
			}
			if (data.board !== void 0) {
				if (!validate58(data.board, {
					instancePath: instancePath + "/board",
					parentData: data,
					parentDataProperty: "board",
					rootData
				})) {
					vErrors = vErrors === null ? validate58.errors : vErrors.concat(validate58.errors);
					errors = vErrors.length;
				}
			}
			if (data.environment !== void 0) {
				let data3 = data.environment;
				if (data3 && typeof data3 == "object" && !Array.isArray(data3)) {
					if (data3.stormCloud === void 0) {
						const err7 = {
							instancePath: instancePath + "/environment",
							schemaPath: "#/$defs/Environment/required",
							keyword: "required",
							params: { missingProperty: "stormCloud" },
							message: "must have required property 'stormCloud'"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					for (const key1 in data3) if (!(key1 === "stormCloud")) {
						const err8 = {
							instancePath: instancePath + "/environment",
							schemaPath: "#/$defs/Environment/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
					if (data3.stormCloud !== void 0) {
						if (typeof data3.stormCloud !== "boolean") {
							const err9 = {
								instancePath: instancePath + "/environment/stormCloud",
								schemaPath: "#/$defs/Environment/properties/stormCloud/type",
								keyword: "type",
								params: { type: "boolean" },
								message: "must be boolean"
							};
							if (vErrors === null) vErrors = [err9];
							else vErrors.push(err9);
							errors++;
						}
					}
				} else {
					const err10 = {
						instancePath: instancePath + "/environment",
						schemaPath: "#/$defs/Environment/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
		} else {
			const err11 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err11];
			else vErrors.push(err11);
			errors++;
		}
		validate57.errors = vErrors;
		return errors === 0;
	}
	var schema87 = { "enum": [
		["human", "cpu"],
		[
			"human",
			"human",
			"cpu"
		],
		[
			"human",
			"cpu",
			"human"
		],
		[
			"human",
			"cpu",
			"cpu"
		],
		[
			"human",
			"human",
			"human",
			"cpu"
		],
		[
			"human",
			"human",
			"cpu",
			"human"
		],
		[
			"human",
			"human",
			"cpu",
			"cpu"
		],
		[
			"human",
			"cpu",
			"human",
			"human"
		],
		[
			"human",
			"cpu",
			"human",
			"cpu"
		],
		[
			"human",
			"cpu",
			"cpu",
			"human"
		],
		[
			"human",
			"cpu",
			"cpu",
			"cpu"
		]
	] };
	function validate61(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.controllers === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "controllers" },
					message: "must have required property 'controllers'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.cpuDifficulty === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "cpuDifficulty" },
					message: "must have required property 'cpuDifficulty'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.board === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "board" },
					message: "must have required property 'board'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.environment === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "environment" },
					message: "must have required property 'environment'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "controllers" || key0 === "cpuDifficulty" || key0 === "board" || key0 === "environment")) {
				const err4 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.controllers !== void 0) {
				let data0 = data.controllers;
				const vSchema0 = schema87.enum;
				if (!(func0(data0, vSchema0[0]) || func0(data0, vSchema0[1]) || func0(data0, vSchema0[2]) || func0(data0, vSchema0[3]) || func0(data0, vSchema0[4]) || func0(data0, vSchema0[5]) || func0(data0, vSchema0[6]) || func0(data0, vSchema0[7]) || func0(data0, vSchema0[8]) || func0(data0, vSchema0[9]) || func0(data0, vSchema0[10]))) {
					const err5 = {
						instancePath: instancePath + "/controllers",
						schemaPath: "#/$defs/MixedControllers/enum",
						keyword: "enum",
						params: { allowedValues: schema87.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.cpuDifficulty !== void 0) {
				let data1 = data.cpuDifficulty;
				if (!(data1 === "easy" || data1 === "standard")) {
					const err6 = {
						instancePath: instancePath + "/cpuDifficulty",
						schemaPath: "#/$defs/Difficulty/enum",
						keyword: "enum",
						params: { allowedValues: schema45.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
			}
			if (data.board !== void 0) {
				if (!validate58(data.board, {
					instancePath: instancePath + "/board",
					parentData: data,
					parentDataProperty: "board",
					rootData
				})) {
					vErrors = vErrors === null ? validate58.errors : vErrors.concat(validate58.errors);
					errors = vErrors.length;
				}
			}
			if (data.environment !== void 0) {
				let data3 = data.environment;
				if (data3 && typeof data3 == "object" && !Array.isArray(data3)) {
					if (data3.stormCloud === void 0) {
						const err7 = {
							instancePath: instancePath + "/environment",
							schemaPath: "#/$defs/Environment/required",
							keyword: "required",
							params: { missingProperty: "stormCloud" },
							message: "must have required property 'stormCloud'"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					for (const key1 in data3) if (!(key1 === "stormCloud")) {
						const err8 = {
							instancePath: instancePath + "/environment",
							schemaPath: "#/$defs/Environment/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
					if (data3.stormCloud !== void 0) {
						if (typeof data3.stormCloud !== "boolean") {
							const err9 = {
								instancePath: instancePath + "/environment/stormCloud",
								schemaPath: "#/$defs/Environment/properties/stormCloud/type",
								keyword: "type",
								params: { type: "boolean" },
								message: "must be boolean"
							};
							if (vErrors === null) vErrors = [err9];
							else vErrors.push(err9);
							errors++;
						}
					}
				} else {
					const err10 = {
						instancePath: instancePath + "/environment",
						schemaPath: "#/$defs/Environment/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
		} else {
			const err11 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err11];
			else vErrors.push(err11);
			errors++;
		}
		validate61.errors = vErrors;
		return errors === 0;
	}
	function validate56(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate57(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate57.errors : vErrors.concat(validate57.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate61(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate61.errors : vErrors.concat(validate61.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs2 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else if (_valid0) {
			valid0 = true;
			passing0 = 1;
		}
		if (!valid0) {
			const err0 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate56.errors = vErrors;
		return errors === 0;
	}
	function validate17(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.roomId === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "roomId" },
					message: "must have required property 'roomId'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.roomCode === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "roomCode" },
					message: "must have required property 'roomCode'"
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
			if (data.revision === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "revision" },
					message: "must have required property 'revision'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.tick === void 0) {
				const err4 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "tick" },
					message: "must have required property 'tick'"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.rulesVersion === void 0) {
				const err5 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "rulesVersion" },
					message: "must have required property 'rulesVersion'"
				};
				if (vErrors === null) vErrors = [err5];
				else vErrors.push(err5);
				errors++;
			}
			if (data.mapId === void 0) {
				const err6 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "mapId" },
					message: "must have required property 'mapId'"
				};
				if (vErrors === null) vErrors = [err6];
				else vErrors.push(err6);
				errors++;
			}
			if (data.roundNumber === void 0) {
				const err7 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "roundNumber" },
					message: "must have required property 'roundNumber'"
				};
				if (vErrors === null) vErrors = [err7];
				else vErrors.push(err7);
				errors++;
			}
			if (data.scores === void 0) {
				const err8 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "scores" },
					message: "must have required property 'scores'"
				};
				if (vErrors === null) vErrors = [err8];
				else vErrors.push(err8);
				errors++;
			}
			if (data.lastRound === void 0) {
				const err9 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "lastRound" },
					message: "must have required property 'lastRound'"
				};
				if (vErrors === null) vErrors = [err9];
				else vErrors.push(err9);
				errors++;
			}
			if (data.seats === void 0) {
				const err10 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "seats" },
					message: "must have required property 'seats'"
				};
				if (vErrors === null) vErrors = [err10];
				else vErrors.push(err10);
				errors++;
			}
			if (data.phase === void 0) {
				const err11 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "phase" },
					message: "must have required property 'phase'"
				};
				if (vErrors === null) vErrors = [err11];
				else vErrors.push(err11);
				errors++;
			}
			if (data.settings === void 0) {
				const err12 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "settings" },
					message: "must have required property 'settings'"
				};
				if (vErrors === null) vErrors = [err12];
				else vErrors.push(err12);
				errors++;
			}
			if (data.settingsRevision === void 0) {
				const err13 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "settingsRevision" },
					message: "must have required property 'settingsRevision'"
				};
				if (vErrors === null) vErrors = [err13];
				else vErrors.push(err13);
				errors++;
			}
			if (data.settingsEditor === void 0) {
				const err14 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "settingsEditor" },
					message: "must have required property 'settingsEditor'"
				};
				if (vErrors === null) vErrors = [err14];
				else vErrors.push(err14);
				errors++;
			}
			if (data.availableItems === void 0) {
				const err15 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "availableItems" },
					message: "must have required property 'availableItems'"
				};
				if (vErrors === null) vErrors = [err15];
				else vErrors.push(err15);
				errors++;
			}
			if (data.configuration === void 0) {
				const err16 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "configuration" },
					message: "must have required property 'configuration'"
				};
				if (vErrors === null) vErrors = [err16];
				else vErrors.push(err16);
				errors++;
			}
			for (const key0 in data) if (!func2.call(schema19.properties, key0)) {
				const err17 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err17];
				else vErrors.push(err17);
				errors++;
			}
			if (data.roomId !== void 0) {
				let data0 = data.roomId;
				if (typeof data0 === "string") {
					if (func3(data0) > 32) {
						const err18 = {
							instancePath: instancePath + "/roomId",
							schemaPath: "#/$defs/RoomId/maxLength",
							keyword: "maxLength",
							params: { limit: 32 },
							message: "must NOT have more than 32 characters"
						};
						if (vErrors === null) vErrors = [err18];
						else vErrors.push(err18);
						errors++;
					}
					if (func3(data0) < 32) {
						const err19 = {
							instancePath: instancePath + "/roomId",
							schemaPath: "#/$defs/RoomId/minLength",
							keyword: "minLength",
							params: { limit: 32 },
							message: "must NOT have fewer than 32 characters"
						};
						if (vErrors === null) vErrors = [err19];
						else vErrors.push(err19);
						errors++;
					}
					if (!pattern0.test(data0)) {
						const err20 = {
							instancePath: instancePath + "/roomId",
							schemaPath: "#/$defs/RoomId/pattern",
							keyword: "pattern",
							params: { pattern: "^[0-9a-f]{32}$" },
							message: "must match pattern \"^[0-9a-f]{32}$\""
						};
						if (vErrors === null) vErrors = [err20];
						else vErrors.push(err20);
						errors++;
					}
				} else {
					const err21 = {
						instancePath: instancePath + "/roomId",
						schemaPath: "#/$defs/RoomId/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err21];
					else vErrors.push(err21);
					errors++;
				}
			}
			if (data.roomCode !== void 0) {
				let data1 = data.roomCode;
				if (typeof data1 === "string") {
					if (func3(data1) > 8) {
						const err22 = {
							instancePath: instancePath + "/roomCode",
							schemaPath: "#/$defs/RoomCode/maxLength",
							keyword: "maxLength",
							params: { limit: 8 },
							message: "must NOT have more than 8 characters"
						};
						if (vErrors === null) vErrors = [err22];
						else vErrors.push(err22);
						errors++;
					}
					if (func3(data1) < 8) {
						const err23 = {
							instancePath: instancePath + "/roomCode",
							schemaPath: "#/$defs/RoomCode/minLength",
							keyword: "minLength",
							params: { limit: 8 },
							message: "must NOT have fewer than 8 characters"
						};
						if (vErrors === null) vErrors = [err23];
						else vErrors.push(err23);
						errors++;
					}
					if (!pattern1.test(data1)) {
						const err24 = {
							instancePath: instancePath + "/roomCode",
							schemaPath: "#/$defs/RoomCode/pattern",
							keyword: "pattern",
							params: { pattern: "^[A-HJ-NP-Z2-9]{8}$" },
							message: "must match pattern \"^[A-HJ-NP-Z2-9]{8}$\""
						};
						if (vErrors === null) vErrors = [err24];
						else vErrors.push(err24);
						errors++;
					}
				} else {
					const err25 = {
						instancePath: instancePath + "/roomCode",
						schemaPath: "#/$defs/RoomCode/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err25];
					else vErrors.push(err25);
					errors++;
				}
			}
			if (data.roundId !== void 0) {
				let data2 = data.roundId;
				if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
					const err26 = {
						instancePath: instancePath + "/roundId",
						schemaPath: "#/$defs/Counter/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err26];
					else vErrors.push(err26);
					errors++;
				}
				if (typeof data2 == "number" && isFinite(data2)) {
					if (data2 > 2147483647 || isNaN(data2)) {
						const err27 = {
							instancePath: instancePath + "/roundId",
							schemaPath: "#/$defs/Counter/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err27];
						else vErrors.push(err27);
						errors++;
					}
					if (data2 < 1 || isNaN(data2)) {
						const err28 = {
							instancePath: instancePath + "/roundId",
							schemaPath: "#/$defs/Counter/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err28];
						else vErrors.push(err28);
						errors++;
					}
				}
			}
			if (data.revision !== void 0) {
				let data3 = data.revision;
				if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3) && isFinite(data3))) {
					const err29 = {
						instancePath: instancePath + "/revision",
						schemaPath: "#/$defs/Counter/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err29];
					else vErrors.push(err29);
					errors++;
				}
				if (typeof data3 == "number" && isFinite(data3)) {
					if (data3 > 2147483647 || isNaN(data3)) {
						const err30 = {
							instancePath: instancePath + "/revision",
							schemaPath: "#/$defs/Counter/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err30];
						else vErrors.push(err30);
						errors++;
					}
					if (data3 < 1 || isNaN(data3)) {
						const err31 = {
							instancePath: instancePath + "/revision",
							schemaPath: "#/$defs/Counter/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err31];
						else vErrors.push(err31);
						errors++;
					}
				}
			}
			if (data.tick !== void 0) {
				let data4 = data.tick;
				if (!(typeof data4 == "number" && !(data4 % 1) && !isNaN(data4) && isFinite(data4))) {
					const err32 = {
						instancePath: instancePath + "/tick",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err32];
					else vErrors.push(err32);
					errors++;
				}
				if (typeof data4 == "number" && isFinite(data4)) {
					if (data4 > 9007199254740991 || isNaN(data4)) {
						const err33 = {
							instancePath: instancePath + "/tick",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err33];
						else vErrors.push(err33);
						errors++;
					}
					if (data4 < 0 || isNaN(data4)) {
						const err34 = {
							instancePath: instancePath + "/tick",
							schemaPath: "#/$defs/Tick/minimum",
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
			if (data.rulesVersion !== void 0) {
				if (7 !== data.rulesVersion) {
					const err35 = {
						instancePath: instancePath + "/rulesVersion",
						schemaPath: "#/$defs/RulesVersion/const",
						keyword: "const",
						params: { allowedValue: 7 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err35];
					else vErrors.push(err35);
					errors++;
				}
			}
			if (data.mapId !== void 0) {
				let data6 = data.mapId;
				if (!(data6 === "fixed_v1" || data6 === "generated_v1" || data6 === "generated_v2")) {
					const err36 = {
						instancePath: instancePath + "/mapId",
						schemaPath: "#/$defs/MapId/enum",
						keyword: "enum",
						params: { allowedValues: schema26.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err36];
					else vErrors.push(err36);
					errors++;
				}
			}
			if (data.roundNumber !== void 0) {
				let data7 = data.roundNumber;
				if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7) && isFinite(data7))) {
					const err37 = {
						instancePath: instancePath + "/roundNumber",
						schemaPath: "#/$defs/RoundCount/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err37];
					else vErrors.push(err37);
					errors++;
				}
				if (typeof data7 == "number" && isFinite(data7)) {
					if (data7 > 2147483647 || isNaN(data7)) {
						const err38 = {
							instancePath: instancePath + "/roundNumber",
							schemaPath: "#/$defs/RoundCount/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err38];
						else vErrors.push(err38);
						errors++;
					}
					if (data7 < 0 || isNaN(data7)) {
						const err39 = {
							instancePath: instancePath + "/roundNumber",
							schemaPath: "#/$defs/RoundCount/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err39];
						else vErrors.push(err39);
						errors++;
					}
				}
			}
			if (data.scores !== void 0) {
				if (!validate18(data.scores, {
					instancePath: instancePath + "/scores",
					parentData: data,
					parentDataProperty: "scores",
					rootData
				})) {
					vErrors = vErrors === null ? validate18.errors : vErrors.concat(validate18.errors);
					errors = vErrors.length;
				}
			}
			if (data.lastRound !== void 0) {
				let data9 = data.lastRound;
				const _errs26 = errors;
				let valid9 = false;
				let passing0 = null;
				const _errs27 = errors;
				if (!validate22(data9, {
					instancePath: instancePath + "/lastRound",
					parentData: data,
					parentDataProperty: "lastRound",
					rootData
				})) {
					vErrors = vErrors === null ? validate22.errors : vErrors.concat(validate22.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs27 === errors;
				if (_valid0) {
					valid9 = true;
					passing0 = 0;
				}
				const _errs28 = errors;
				if (data9 !== null) {
					const err40 = {
						instancePath: instancePath + "/lastRound",
						schemaPath: "#/properties/lastRound/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err40];
					else vErrors.push(err40);
					errors++;
				}
				var _valid0 = _errs28 === errors;
				if (_valid0 && valid9) {
					valid9 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid9 = true;
					passing0 = 1;
				}
				if (!valid9) {
					const err41 = {
						instancePath: instancePath + "/lastRound",
						schemaPath: "#/properties/lastRound/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err41];
					else vErrors.push(err41);
					errors++;
				} else {
					errors = _errs26;
					if (vErrors !== null) {
						if (_errs26) vErrors.length = _errs26;
						else vErrors = null;
					}
				}
			}
			if (data.seats !== void 0) {
				let data10 = data.seats;
				if (Array.isArray(data10)) {
					if (data10.length > 4) {
						const err42 = {
							instancePath: instancePath + "/seats",
							schemaPath: "#/properties/seats/maxItems",
							keyword: "maxItems",
							params: { limit: 4 },
							message: "must NOT have more than 4 items"
						};
						if (vErrors === null) vErrors = [err42];
						else vErrors.push(err42);
						errors++;
					}
					if (data10.length < 2) {
						const err43 = {
							instancePath: instancePath + "/seats",
							schemaPath: "#/properties/seats/minItems",
							keyword: "minItems",
							params: { limit: 2 },
							message: "must NOT have fewer than 2 items"
						};
						if (vErrors === null) vErrors = [err43];
						else vErrors.push(err43);
						errors++;
					}
					const len0 = data10.length;
					for (let i0 = 0; i0 < len0; i0++) if (!validate28(data10[i0], {
						instancePath: instancePath + "/seats/" + i0,
						parentData: data10,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate28.errors : vErrors.concat(validate28.errors);
						errors = vErrors.length;
					}
				} else {
					const err44 = {
						instancePath: instancePath + "/seats",
						schemaPath: "#/properties/seats/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err44];
					else vErrors.push(err44);
					errors++;
				}
			}
			if (data.phase !== void 0) {
				if (!validate38(data.phase, {
					instancePath: instancePath + "/phase",
					parentData: data,
					parentDataProperty: "phase",
					rootData
				})) {
					vErrors = vErrors === null ? validate38.errors : vErrors.concat(validate38.errors);
					errors = vErrors.length;
				}
			}
			if (data.settings !== void 0) {
				if (!validate54(data.settings, {
					instancePath: instancePath + "/settings",
					parentData: data,
					parentDataProperty: "settings",
					rootData
				})) {
					vErrors = vErrors === null ? validate54.errors : vErrors.concat(validate54.errors);
					errors = vErrors.length;
				}
			}
			if (data.settingsRevision !== void 0) {
				let data14 = data.settingsRevision;
				if (!(typeof data14 == "number" && !(data14 % 1) && !isNaN(data14) && isFinite(data14))) {
					const err45 = {
						instancePath: instancePath + "/settingsRevision",
						schemaPath: "#/$defs/Counter/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err45];
					else vErrors.push(err45);
					errors++;
				}
				if (typeof data14 == "number" && isFinite(data14)) {
					if (data14 > 2147483647 || isNaN(data14)) {
						const err46 = {
							instancePath: instancePath + "/settingsRevision",
							schemaPath: "#/$defs/Counter/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err46];
						else vErrors.push(err46);
						errors++;
					}
					if (data14 < 1 || isNaN(data14)) {
						const err47 = {
							instancePath: instancePath + "/settingsRevision",
							schemaPath: "#/$defs/Counter/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err47];
						else vErrors.push(err47);
						errors++;
					}
				}
			}
			if (data.settingsEditor !== void 0) {
				let data15 = data.settingsEditor;
				const _errs39 = errors;
				let valid13 = false;
				let passing1 = null;
				const _errs40 = errors;
				if (!(data15 === "one" || data15 === "two" || data15 === "three" || data15 === "four")) {
					const err48 = {
						instancePath: instancePath + "/settingsEditor",
						schemaPath: "#/$defs/Seat/enum",
						keyword: "enum",
						params: { allowedValues: schema30.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err48];
					else vErrors.push(err48);
					errors++;
				}
				var _valid1 = _errs40 === errors;
				if (_valid1) {
					valid13 = true;
					passing1 = 0;
				}
				const _errs42 = errors;
				if (data15 !== null) {
					const err49 = {
						instancePath: instancePath + "/settingsEditor",
						schemaPath: "#/properties/settingsEditor/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err49];
					else vErrors.push(err49);
					errors++;
				}
				var _valid1 = _errs42 === errors;
				if (_valid1 && valid13) {
					valid13 = false;
					passing1 = [passing1, 1];
				} else if (_valid1) {
					valid13 = true;
					passing1 = 1;
				}
				if (!valid13) {
					const err50 = {
						instancePath: instancePath + "/settingsEditor",
						schemaPath: "#/properties/settingsEditor/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing1 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err50];
					else vErrors.push(err50);
					errors++;
				} else {
					errors = _errs39;
					if (vErrors !== null) {
						if (_errs39) vErrors.length = _errs39;
						else vErrors = null;
					}
				}
			}
			if (data.availableItems !== void 0) {
				let data16 = data.availableItems;
				if (Array.isArray(data16)) {
					if (data16.length > 22) {
						const err51 = {
							instancePath: instancePath + "/availableItems",
							schemaPath: "#/properties/availableItems/maxItems",
							keyword: "maxItems",
							params: { limit: 22 },
							message: "must NOT have more than 22 items"
						};
						if (vErrors === null) vErrors = [err51];
						else vErrors.push(err51);
						errors++;
					}
					if (data16.length < 0) {
						const err52 = {
							instancePath: instancePath + "/availableItems",
							schemaPath: "#/properties/availableItems/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err52];
						else vErrors.push(err52);
						errors++;
					}
					const len1 = data16.length;
					for (let i1 = 0; i1 < len1; i1++) {
						let data17 = data16[i1];
						if (!(data17 === "kick" || data17 === "punch" || data17 === "bracelet" || data17 === "wallPass" || data17 === "heart" || data17 === "remoteBomb" || data17 === "piercingBomb" || data17 === "powerBomb" || data17 === "landmine" || data17 === "shield" || data17 === "stopwatch" || data17 === "invisibilityCloak" || data17 === "goldenFlame" || data17 === "shortFuse" || data17 === "diarrhea" || data17 === "slowFoot" || data17 === "shortFlame" || data17 === "longFuse" || data17 === "reverseControls" || data17 === "invisibleBombs" || data17 === "oilBarrel" || data17 === "teleporter")) {
							const err53 = {
								instancePath: instancePath + "/availableItems/" + i1,
								schemaPath: "#/$defs/ItemId/enum",
								keyword: "enum",
								params: { allowedValues: schema73.enum },
								message: "must be equal to one of the allowed values"
							};
							if (vErrors === null) vErrors = [err53];
							else vErrors.push(err53);
							errors++;
						}
					}
					let i2 = data16.length;
					let j0;
					if (i2 > 1) {
						outer0: for (; i2--;) for (j0 = i2; j0--;) if (func0(data16[i2], data16[j0])) {
							const err54 = {
								instancePath: instancePath + "/availableItems",
								schemaPath: "#/properties/availableItems/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i2,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i2 + " are identical)"
							};
							if (vErrors === null) vErrors = [err54];
							else vErrors.push(err54);
							errors++;
							break outer0;
						}
					}
				} else {
					const err55 = {
						instancePath: instancePath + "/availableItems",
						schemaPath: "#/properties/availableItems/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err55];
					else vErrors.push(err55);
					errors++;
				}
			}
			if (data.configuration !== void 0) {
				if (!validate56(data.configuration, {
					instancePath: instancePath + "/configuration",
					parentData: data,
					parentDataProperty: "configuration",
					rootData
				})) {
					vErrors = vErrors === null ? validate56.errors : vErrors.concat(validate56.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err56 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err56];
			else vErrors.push(err56);
			errors++;
		}
		validate17.errors = vErrors;
		return errors === 0;
	}
	var schema92 = {
		"type": "object",
		"additionalProperties": false,
		"required": [
			"runningUpdates",
			"remainingTicks",
			"physicalTick",
			"arena",
			"actors",
			"bombs",
			"flames",
			"oilBarrels",
			"pickups",
			"stormCloud"
		],
		"properties": {
			"runningUpdates": {
				"type": "integer",
				"minimum": 0,
				"maximum": 10800
			},
			"remainingTicks": {
				"type": "integer",
				"minimum": 0,
				"maximum": 10800
			},
			"physicalTick": { "$ref": "#/$defs/Tick" },
			"arena": { "$ref": "#/$defs/ArenaView" },
			"actors": {
				"type": "array",
				"minItems": 2,
				"maxItems": 4,
				"items": { "$ref": "#/$defs/ActorView" }
			},
			"bombs": {
				"type": "array",
				"minItems": 0,
				"maxItems": 32,
				"items": { "$ref": "#/$defs/BombView" }
			},
			"flames": {
				"type": "array",
				"minItems": 0,
				"maxItems": 195,
				"items": { "$ref": "#/$defs/FlameView" }
			},
			"pickups": {
				"type": "array",
				"minItems": 0,
				"maxItems": 195,
				"uniqueItems": true,
				"items": { "$ref": "#/$defs/PickupView" }
			},
			"oilBarrels": {
				"type": "array",
				"minItems": 0,
				"maxItems": 8,
				"uniqueItems": true,
				"items": { "$ref": "#/$defs/OilBarrelView" }
			},
			"stormCloud": { "oneOf": [{ "$ref": "#/$defs/StormCloudView" }, { "type": "null" }] }
		}
	};
	var schema96 = { "enum": ["floor", "wall"] };
	function validate68(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.width === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "width" },
					message: "must have required property 'width'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.height === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "height" },
					message: "must have required property 'height'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.terrain === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "terrain" },
					message: "must have required property 'terrain'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.blocks === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "blocks" },
					message: "must have required property 'blocks'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "width" || key0 === "height" || key0 === "terrain" || key0 === "blocks")) {
				const err4 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.width !== void 0) {
				if (11 !== data.width) {
					const err5 = {
						instancePath: instancePath + "/width",
						schemaPath: "#/properties/width/const",
						keyword: "const",
						params: { allowedValue: 11 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.height !== void 0) {
				if (9 !== data.height) {
					const err6 = {
						instancePath: instancePath + "/height",
						schemaPath: "#/properties/height/const",
						keyword: "const",
						params: { allowedValue: 9 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
			}
			if (data.terrain !== void 0) {
				let data2 = data.terrain;
				if (Array.isArray(data2)) {
					if (data2.length > 99) {
						const err7 = {
							instancePath: instancePath + "/terrain",
							schemaPath: "#/properties/terrain/maxItems",
							keyword: "maxItems",
							params: { limit: 99 },
							message: "must NOT have more than 99 items"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data2.length < 99) {
						const err8 = {
							instancePath: instancePath + "/terrain",
							schemaPath: "#/properties/terrain/minItems",
							keyword: "minItems",
							params: { limit: 99 },
							message: "must NOT have fewer than 99 items"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
					const len0 = data2.length;
					for (let i0 = 0; i0 < len0; i0++) {
						let data3 = data2[i0];
						if (!(data3 === "floor" || data3 === "wall")) {
							const err9 = {
								instancePath: instancePath + "/terrain/" + i0,
								schemaPath: "#/$defs/Terrain/enum",
								keyword: "enum",
								params: { allowedValues: schema96.enum },
								message: "must be equal to one of the allowed values"
							};
							if (vErrors === null) vErrors = [err9];
							else vErrors.push(err9);
							errors++;
						}
					}
				} else {
					const err10 = {
						instancePath: instancePath + "/terrain",
						schemaPath: "#/properties/terrain/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
			if (data.blocks !== void 0) {
				let data4 = data.blocks;
				if (Array.isArray(data4)) {
					if (data4.length > 99) {
						const err11 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/maxItems",
							keyword: "maxItems",
							params: { limit: 99 },
							message: "must NOT have more than 99 items"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
					if (data4.length < 0) {
						const err12 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err12];
						else vErrors.push(err12);
						errors++;
					}
					const len1 = data4.length;
					for (let i1 = 0; i1 < len1; i1++) {
						let data5 = data4[i1];
						if (data5 && typeof data5 == "object" && !Array.isArray(data5)) {
							if (data5.column === void 0) {
								const err13 = {
									instancePath: instancePath + "/blocks/" + i1,
									schemaPath: "#/$defs/Cell/required",
									keyword: "required",
									params: { missingProperty: "column" },
									message: "must have required property 'column'"
								};
								if (vErrors === null) vErrors = [err13];
								else vErrors.push(err13);
								errors++;
							}
							if (data5.row === void 0) {
								const err14 = {
									instancePath: instancePath + "/blocks/" + i1,
									schemaPath: "#/$defs/Cell/required",
									keyword: "required",
									params: { missingProperty: "row" },
									message: "must have required property 'row'"
								};
								if (vErrors === null) vErrors = [err14];
								else vErrors.push(err14);
								errors++;
							}
							for (const key1 in data5) if (!(key1 === "column" || key1 === "row")) {
								const err15 = {
									instancePath: instancePath + "/blocks/" + i1,
									schemaPath: "#/$defs/Cell/additionalProperties",
									keyword: "additionalProperties",
									params: { additionalProperty: key1 },
									message: "must NOT have additional properties"
								};
								if (vErrors === null) vErrors = [err15];
								else vErrors.push(err15);
								errors++;
							}
							if (data5.column !== void 0) {
								let data6 = data5.column;
								if (!(typeof data6 == "number" && !(data6 % 1) && !isNaN(data6) && isFinite(data6))) {
									const err16 = {
										instancePath: instancePath + "/blocks/" + i1 + "/column",
										schemaPath: "#/$defs/Cell/properties/column/type",
										keyword: "type",
										params: { type: "integer" },
										message: "must be integer"
									};
									if (vErrors === null) vErrors = [err16];
									else vErrors.push(err16);
									errors++;
								}
								if (typeof data6 == "number" && isFinite(data6)) {
									if (data6 > 14 || isNaN(data6)) {
										const err17 = {
											instancePath: instancePath + "/blocks/" + i1 + "/column",
											schemaPath: "#/$defs/Cell/properties/column/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 14
											},
											message: "must be <= 14"
										};
										if (vErrors === null) vErrors = [err17];
										else vErrors.push(err17);
										errors++;
									}
									if (data6 < 0 || isNaN(data6)) {
										const err18 = {
											instancePath: instancePath + "/blocks/" + i1 + "/column",
											schemaPath: "#/$defs/Cell/properties/column/minimum",
											keyword: "minimum",
											params: {
												comparison: ">=",
												limit: 0
											},
											message: "must be >= 0"
										};
										if (vErrors === null) vErrors = [err18];
										else vErrors.push(err18);
										errors++;
									}
								}
							}
							if (data5.row !== void 0) {
								let data7 = data5.row;
								if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7) && isFinite(data7))) {
									const err19 = {
										instancePath: instancePath + "/blocks/" + i1 + "/row",
										schemaPath: "#/$defs/Cell/properties/row/type",
										keyword: "type",
										params: { type: "integer" },
										message: "must be integer"
									};
									if (vErrors === null) vErrors = [err19];
									else vErrors.push(err19);
									errors++;
								}
								if (typeof data7 == "number" && isFinite(data7)) {
									if (data7 > 12 || isNaN(data7)) {
										const err20 = {
											instancePath: instancePath + "/blocks/" + i1 + "/row",
											schemaPath: "#/$defs/Cell/properties/row/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 12
											},
											message: "must be <= 12"
										};
										if (vErrors === null) vErrors = [err20];
										else vErrors.push(err20);
										errors++;
									}
									if (data7 < 0 || isNaN(data7)) {
										const err21 = {
											instancePath: instancePath + "/blocks/" + i1 + "/row",
											schemaPath: "#/$defs/Cell/properties/row/minimum",
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
						} else {
							const err22 = {
								instancePath: instancePath + "/blocks/" + i1,
								schemaPath: "#/$defs/Cell/type",
								keyword: "type",
								params: { type: "object" },
								message: "must be object"
							};
							if (vErrors === null) vErrors = [err22];
							else vErrors.push(err22);
							errors++;
						}
					}
					let i2 = data4.length;
					let j0;
					if (i2 > 1) {
						outer0: for (; i2--;) for (j0 = i2; j0--;) if (func0(data4[i2], data4[j0])) {
							const err23 = {
								instancePath: instancePath + "/blocks",
								schemaPath: "#/properties/blocks/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i2,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i2 + " are identical)"
							};
							if (vErrors === null) vErrors = [err23];
							else vErrors.push(err23);
							errors++;
							break outer0;
						}
					}
				} else {
					const err24 = {
						instancePath: instancePath + "/blocks",
						schemaPath: "#/properties/blocks/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err24];
					else vErrors.push(err24);
					errors++;
				}
			}
		} else {
			const err25 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err25];
			else vErrors.push(err25);
			errors++;
		}
		validate68.errors = vErrors;
		return errors === 0;
	}
	function validate70(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.width === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "width" },
					message: "must have required property 'width'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.height === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "height" },
					message: "must have required property 'height'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.terrain === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "terrain" },
					message: "must have required property 'terrain'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.blocks === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "blocks" },
					message: "must have required property 'blocks'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "width" || key0 === "height" || key0 === "terrain" || key0 === "blocks")) {
				const err4 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.width !== void 0) {
				if (13 !== data.width) {
					const err5 = {
						instancePath: instancePath + "/width",
						schemaPath: "#/properties/width/const",
						keyword: "const",
						params: { allowedValue: 13 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.height !== void 0) {
				if (11 !== data.height) {
					const err6 = {
						instancePath: instancePath + "/height",
						schemaPath: "#/properties/height/const",
						keyword: "const",
						params: { allowedValue: 11 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
			}
			if (data.terrain !== void 0) {
				let data2 = data.terrain;
				if (Array.isArray(data2)) {
					if (data2.length > 143) {
						const err7 = {
							instancePath: instancePath + "/terrain",
							schemaPath: "#/properties/terrain/maxItems",
							keyword: "maxItems",
							params: { limit: 143 },
							message: "must NOT have more than 143 items"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data2.length < 143) {
						const err8 = {
							instancePath: instancePath + "/terrain",
							schemaPath: "#/properties/terrain/minItems",
							keyword: "minItems",
							params: { limit: 143 },
							message: "must NOT have fewer than 143 items"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
					const len0 = data2.length;
					for (let i0 = 0; i0 < len0; i0++) {
						let data3 = data2[i0];
						if (!(data3 === "floor" || data3 === "wall")) {
							const err9 = {
								instancePath: instancePath + "/terrain/" + i0,
								schemaPath: "#/$defs/Terrain/enum",
								keyword: "enum",
								params: { allowedValues: schema96.enum },
								message: "must be equal to one of the allowed values"
							};
							if (vErrors === null) vErrors = [err9];
							else vErrors.push(err9);
							errors++;
						}
					}
				} else {
					const err10 = {
						instancePath: instancePath + "/terrain",
						schemaPath: "#/properties/terrain/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
			if (data.blocks !== void 0) {
				let data4 = data.blocks;
				if (Array.isArray(data4)) {
					if (data4.length > 143) {
						const err11 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/maxItems",
							keyword: "maxItems",
							params: { limit: 143 },
							message: "must NOT have more than 143 items"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
					if (data4.length < 0) {
						const err12 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err12];
						else vErrors.push(err12);
						errors++;
					}
					const len1 = data4.length;
					for (let i1 = 0; i1 < len1; i1++) {
						let data5 = data4[i1];
						if (data5 && typeof data5 == "object" && !Array.isArray(data5)) {
							if (data5.column === void 0) {
								const err13 = {
									instancePath: instancePath + "/blocks/" + i1,
									schemaPath: "#/$defs/Cell/required",
									keyword: "required",
									params: { missingProperty: "column" },
									message: "must have required property 'column'"
								};
								if (vErrors === null) vErrors = [err13];
								else vErrors.push(err13);
								errors++;
							}
							if (data5.row === void 0) {
								const err14 = {
									instancePath: instancePath + "/blocks/" + i1,
									schemaPath: "#/$defs/Cell/required",
									keyword: "required",
									params: { missingProperty: "row" },
									message: "must have required property 'row'"
								};
								if (vErrors === null) vErrors = [err14];
								else vErrors.push(err14);
								errors++;
							}
							for (const key1 in data5) if (!(key1 === "column" || key1 === "row")) {
								const err15 = {
									instancePath: instancePath + "/blocks/" + i1,
									schemaPath: "#/$defs/Cell/additionalProperties",
									keyword: "additionalProperties",
									params: { additionalProperty: key1 },
									message: "must NOT have additional properties"
								};
								if (vErrors === null) vErrors = [err15];
								else vErrors.push(err15);
								errors++;
							}
							if (data5.column !== void 0) {
								let data6 = data5.column;
								if (!(typeof data6 == "number" && !(data6 % 1) && !isNaN(data6) && isFinite(data6))) {
									const err16 = {
										instancePath: instancePath + "/blocks/" + i1 + "/column",
										schemaPath: "#/$defs/Cell/properties/column/type",
										keyword: "type",
										params: { type: "integer" },
										message: "must be integer"
									};
									if (vErrors === null) vErrors = [err16];
									else vErrors.push(err16);
									errors++;
								}
								if (typeof data6 == "number" && isFinite(data6)) {
									if (data6 > 14 || isNaN(data6)) {
										const err17 = {
											instancePath: instancePath + "/blocks/" + i1 + "/column",
											schemaPath: "#/$defs/Cell/properties/column/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 14
											},
											message: "must be <= 14"
										};
										if (vErrors === null) vErrors = [err17];
										else vErrors.push(err17);
										errors++;
									}
									if (data6 < 0 || isNaN(data6)) {
										const err18 = {
											instancePath: instancePath + "/blocks/" + i1 + "/column",
											schemaPath: "#/$defs/Cell/properties/column/minimum",
											keyword: "minimum",
											params: {
												comparison: ">=",
												limit: 0
											},
											message: "must be >= 0"
										};
										if (vErrors === null) vErrors = [err18];
										else vErrors.push(err18);
										errors++;
									}
								}
							}
							if (data5.row !== void 0) {
								let data7 = data5.row;
								if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7) && isFinite(data7))) {
									const err19 = {
										instancePath: instancePath + "/blocks/" + i1 + "/row",
										schemaPath: "#/$defs/Cell/properties/row/type",
										keyword: "type",
										params: { type: "integer" },
										message: "must be integer"
									};
									if (vErrors === null) vErrors = [err19];
									else vErrors.push(err19);
									errors++;
								}
								if (typeof data7 == "number" && isFinite(data7)) {
									if (data7 > 12 || isNaN(data7)) {
										const err20 = {
											instancePath: instancePath + "/blocks/" + i1 + "/row",
											schemaPath: "#/$defs/Cell/properties/row/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 12
											},
											message: "must be <= 12"
										};
										if (vErrors === null) vErrors = [err20];
										else vErrors.push(err20);
										errors++;
									}
									if (data7 < 0 || isNaN(data7)) {
										const err21 = {
											instancePath: instancePath + "/blocks/" + i1 + "/row",
											schemaPath: "#/$defs/Cell/properties/row/minimum",
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
						} else {
							const err22 = {
								instancePath: instancePath + "/blocks/" + i1,
								schemaPath: "#/$defs/Cell/type",
								keyword: "type",
								params: { type: "object" },
								message: "must be object"
							};
							if (vErrors === null) vErrors = [err22];
							else vErrors.push(err22);
							errors++;
						}
					}
					let i2 = data4.length;
					let j0;
					if (i2 > 1) {
						outer0: for (; i2--;) for (j0 = i2; j0--;) if (func0(data4[i2], data4[j0])) {
							const err23 = {
								instancePath: instancePath + "/blocks",
								schemaPath: "#/properties/blocks/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i2,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i2 + " are identical)"
							};
							if (vErrors === null) vErrors = [err23];
							else vErrors.push(err23);
							errors++;
							break outer0;
						}
					}
				} else {
					const err24 = {
						instancePath: instancePath + "/blocks",
						schemaPath: "#/properties/blocks/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err24];
					else vErrors.push(err24);
					errors++;
				}
			}
		} else {
			const err25 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err25];
			else vErrors.push(err25);
			errors++;
		}
		validate70.errors = vErrors;
		return errors === 0;
	}
	function validate72(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.width === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "width" },
					message: "must have required property 'width'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.height === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "height" },
					message: "must have required property 'height'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.terrain === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "terrain" },
					message: "must have required property 'terrain'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.blocks === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "blocks" },
					message: "must have required property 'blocks'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "width" || key0 === "height" || key0 === "terrain" || key0 === "blocks")) {
				const err4 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.width !== void 0) {
				if (15 !== data.width) {
					const err5 = {
						instancePath: instancePath + "/width",
						schemaPath: "#/properties/width/const",
						keyword: "const",
						params: { allowedValue: 15 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.height !== void 0) {
				if (13 !== data.height) {
					const err6 = {
						instancePath: instancePath + "/height",
						schemaPath: "#/properties/height/const",
						keyword: "const",
						params: { allowedValue: 13 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
			}
			if (data.terrain !== void 0) {
				let data2 = data.terrain;
				if (Array.isArray(data2)) {
					if (data2.length > 195) {
						const err7 = {
							instancePath: instancePath + "/terrain",
							schemaPath: "#/properties/terrain/maxItems",
							keyword: "maxItems",
							params: { limit: 195 },
							message: "must NOT have more than 195 items"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data2.length < 195) {
						const err8 = {
							instancePath: instancePath + "/terrain",
							schemaPath: "#/properties/terrain/minItems",
							keyword: "minItems",
							params: { limit: 195 },
							message: "must NOT have fewer than 195 items"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
					const len0 = data2.length;
					for (let i0 = 0; i0 < len0; i0++) {
						let data3 = data2[i0];
						if (!(data3 === "floor" || data3 === "wall")) {
							const err9 = {
								instancePath: instancePath + "/terrain/" + i0,
								schemaPath: "#/$defs/Terrain/enum",
								keyword: "enum",
								params: { allowedValues: schema96.enum },
								message: "must be equal to one of the allowed values"
							};
							if (vErrors === null) vErrors = [err9];
							else vErrors.push(err9);
							errors++;
						}
					}
				} else {
					const err10 = {
						instancePath: instancePath + "/terrain",
						schemaPath: "#/properties/terrain/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
			if (data.blocks !== void 0) {
				let data4 = data.blocks;
				if (Array.isArray(data4)) {
					if (data4.length > 195) {
						const err11 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/maxItems",
							keyword: "maxItems",
							params: { limit: 195 },
							message: "must NOT have more than 195 items"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
					if (data4.length < 0) {
						const err12 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err12];
						else vErrors.push(err12);
						errors++;
					}
					const len1 = data4.length;
					for (let i1 = 0; i1 < len1; i1++) {
						let data5 = data4[i1];
						if (data5 && typeof data5 == "object" && !Array.isArray(data5)) {
							if (data5.column === void 0) {
								const err13 = {
									instancePath: instancePath + "/blocks/" + i1,
									schemaPath: "#/$defs/Cell/required",
									keyword: "required",
									params: { missingProperty: "column" },
									message: "must have required property 'column'"
								};
								if (vErrors === null) vErrors = [err13];
								else vErrors.push(err13);
								errors++;
							}
							if (data5.row === void 0) {
								const err14 = {
									instancePath: instancePath + "/blocks/" + i1,
									schemaPath: "#/$defs/Cell/required",
									keyword: "required",
									params: { missingProperty: "row" },
									message: "must have required property 'row'"
								};
								if (vErrors === null) vErrors = [err14];
								else vErrors.push(err14);
								errors++;
							}
							for (const key1 in data5) if (!(key1 === "column" || key1 === "row")) {
								const err15 = {
									instancePath: instancePath + "/blocks/" + i1,
									schemaPath: "#/$defs/Cell/additionalProperties",
									keyword: "additionalProperties",
									params: { additionalProperty: key1 },
									message: "must NOT have additional properties"
								};
								if (vErrors === null) vErrors = [err15];
								else vErrors.push(err15);
								errors++;
							}
							if (data5.column !== void 0) {
								let data6 = data5.column;
								if (!(typeof data6 == "number" && !(data6 % 1) && !isNaN(data6) && isFinite(data6))) {
									const err16 = {
										instancePath: instancePath + "/blocks/" + i1 + "/column",
										schemaPath: "#/$defs/Cell/properties/column/type",
										keyword: "type",
										params: { type: "integer" },
										message: "must be integer"
									};
									if (vErrors === null) vErrors = [err16];
									else vErrors.push(err16);
									errors++;
								}
								if (typeof data6 == "number" && isFinite(data6)) {
									if (data6 > 14 || isNaN(data6)) {
										const err17 = {
											instancePath: instancePath + "/blocks/" + i1 + "/column",
											schemaPath: "#/$defs/Cell/properties/column/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 14
											},
											message: "must be <= 14"
										};
										if (vErrors === null) vErrors = [err17];
										else vErrors.push(err17);
										errors++;
									}
									if (data6 < 0 || isNaN(data6)) {
										const err18 = {
											instancePath: instancePath + "/blocks/" + i1 + "/column",
											schemaPath: "#/$defs/Cell/properties/column/minimum",
											keyword: "minimum",
											params: {
												comparison: ">=",
												limit: 0
											},
											message: "must be >= 0"
										};
										if (vErrors === null) vErrors = [err18];
										else vErrors.push(err18);
										errors++;
									}
								}
							}
							if (data5.row !== void 0) {
								let data7 = data5.row;
								if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7) && isFinite(data7))) {
									const err19 = {
										instancePath: instancePath + "/blocks/" + i1 + "/row",
										schemaPath: "#/$defs/Cell/properties/row/type",
										keyword: "type",
										params: { type: "integer" },
										message: "must be integer"
									};
									if (vErrors === null) vErrors = [err19];
									else vErrors.push(err19);
									errors++;
								}
								if (typeof data7 == "number" && isFinite(data7)) {
									if (data7 > 12 || isNaN(data7)) {
										const err20 = {
											instancePath: instancePath + "/blocks/" + i1 + "/row",
											schemaPath: "#/$defs/Cell/properties/row/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 12
											},
											message: "must be <= 12"
										};
										if (vErrors === null) vErrors = [err20];
										else vErrors.push(err20);
										errors++;
									}
									if (data7 < 0 || isNaN(data7)) {
										const err21 = {
											instancePath: instancePath + "/blocks/" + i1 + "/row",
											schemaPath: "#/$defs/Cell/properties/row/minimum",
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
						} else {
							const err22 = {
								instancePath: instancePath + "/blocks/" + i1,
								schemaPath: "#/$defs/Cell/type",
								keyword: "type",
								params: { type: "object" },
								message: "must be object"
							};
							if (vErrors === null) vErrors = [err22];
							else vErrors.push(err22);
							errors++;
						}
					}
					let i2 = data4.length;
					let j0;
					if (i2 > 1) {
						outer0: for (; i2--;) for (j0 = i2; j0--;) if (func0(data4[i2], data4[j0])) {
							const err23 = {
								instancePath: instancePath + "/blocks",
								schemaPath: "#/properties/blocks/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i2,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i2 + " are identical)"
							};
							if (vErrors === null) vErrors = [err23];
							else vErrors.push(err23);
							errors++;
							break outer0;
						}
					}
				} else {
					const err24 = {
						instancePath: instancePath + "/blocks",
						schemaPath: "#/properties/blocks/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err24];
					else vErrors.push(err24);
					errors++;
				}
			}
		} else {
			const err25 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err25];
			else vErrors.push(err25);
			errors++;
		}
		validate72.errors = vErrors;
		return errors === 0;
	}
	function validate67(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate68(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate68.errors : vErrors.concat(validate68.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate70(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate70.errors : vErrors.concat(validate70.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs2 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else {
			if (_valid0) {
				valid0 = true;
				passing0 = 1;
			}
			const _errs3 = errors;
			if (!validate72(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate72.errors : vErrors.concat(validate72.errors);
				errors = vErrors.length;
			}
			var _valid0 = _errs3 === errors;
			if (_valid0 && valid0) {
				valid0 = false;
				passing0 = [passing0, 2];
			} else if (_valid0) {
				valid0 = true;
				passing0 = 2;
			}
		}
		if (!valid0) {
			const err0 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate67.errors = vErrors;
		return errors === 0;
	}
	var schema105 = {
		"type": "object",
		"additionalProperties": false,
		"required": [
			"type",
			"id",
			"seat",
			"position",
			"heading",
			"speedLevel",
			"alive",
			"bombCapacity",
			"bombRange",
			"liveBombCount",
			"inventory",
			"effects",
			"damageGraceRemainingTicks",
			"voluntaryActionsFrozen",
			"lastTeleportTick",
			"stormSlowRemainingTicks"
		],
		"properties": {
			"type": { "const": "visible" },
			"id": { "$ref": "#/$defs/EntityId" },
			"seat": { "$ref": "#/$defs/Seat" },
			"position": { "$ref": "#/$defs/Position" },
			"heading": { "$ref": "#/$defs/Direction" },
			"speedLevel": {
				"type": "integer",
				"minimum": 0,
				"maximum": 4
			},
			"alive": { "type": "boolean" },
			"bombCapacity": {
				"type": "integer",
				"minimum": 1,
				"maximum": 8
			},
			"bombRange": {
				"type": "integer",
				"minimum": 1,
				"maximum": 8
			},
			"liveBombCount": {
				"type": "integer",
				"minimum": 0,
				"maximum": 8
			},
			"inventory": { "$ref": "#/$defs/InventoryView" },
			"effects": {
				"type": "array",
				"minItems": 0,
				"maxItems": 16,
				"items": { "$ref": "#/$defs/EffectView" }
			},
			"damageGraceRemainingTicks": {
				"type": "integer",
				"minimum": 0,
				"maximum": 60
			},
			"voluntaryActionsFrozen": { "type": "boolean" },
			"lastTeleportTick": { "oneOf": [{ "$ref": "#/$defs/Tick" }, { "type": "null" }] },
			"stormSlowRemainingTicks": {
				"type": "integer",
				"minimum": 0,
				"maximum": 180
			}
		}
	};
	var schema109 = { "enum": [
		"north",
		"east",
		"south",
		"west"
	] };
	var schema111 = { "enum": [
		"kick",
		"punch",
		"bracelet",
		"wallPass",
		"heart"
	] };
	var schema112 = { "enum": [
		"remoteBomb",
		"piercingBomb",
		"powerBomb",
		"landmine"
	] };
	function validate77(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.equipment === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "equipment" },
					message: "must have required property 'equipment'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.weapon === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "weapon" },
					message: "must have required property 'weapon'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "equipment" || key0 === "weapon")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.equipment !== void 0) {
				let data0 = data.equipment;
				const _errs3 = errors;
				let valid1 = false;
				let passing0 = null;
				const _errs4 = errors;
				if (!(data0 === "kick" || data0 === "punch" || data0 === "bracelet" || data0 === "wallPass" || data0 === "heart")) {
					const err3 = {
						instancePath: instancePath + "/equipment",
						schemaPath: "#/$defs/EquipmentId/enum",
						keyword: "enum",
						params: { allowedValues: schema111.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
				var _valid0 = _errs4 === errors;
				if (_valid0) {
					valid1 = true;
					passing0 = 0;
				}
				const _errs6 = errors;
				if (data0 !== null) {
					const err4 = {
						instancePath: instancePath + "/equipment",
						schemaPath: "#/properties/equipment/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				var _valid0 = _errs6 === errors;
				if (_valid0 && valid1) {
					valid1 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid1 = true;
					passing0 = 1;
				}
				if (!valid1) {
					const err5 = {
						instancePath: instancePath + "/equipment",
						schemaPath: "#/properties/equipment/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				} else {
					errors = _errs3;
					if (vErrors !== null) {
						if (_errs3) vErrors.length = _errs3;
						else vErrors = null;
					}
				}
			}
			if (data.weapon !== void 0) {
				let data1 = data.weapon;
				const _errs9 = errors;
				let valid3 = false;
				let passing1 = null;
				const _errs10 = errors;
				if (!(data1 === "remoteBomb" || data1 === "piercingBomb" || data1 === "powerBomb" || data1 === "landmine")) {
					const err6 = {
						instancePath: instancePath + "/weapon",
						schemaPath: "#/$defs/WeaponId/enum",
						keyword: "enum",
						params: { allowedValues: schema112.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
				var _valid1 = _errs10 === errors;
				if (_valid1) {
					valid3 = true;
					passing1 = 0;
				}
				const _errs12 = errors;
				if (data1 !== null) {
					const err7 = {
						instancePath: instancePath + "/weapon",
						schemaPath: "#/properties/weapon/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err7];
					else vErrors.push(err7);
					errors++;
				}
				var _valid1 = _errs12 === errors;
				if (_valid1 && valid3) {
					valid3 = false;
					passing1 = [passing1, 1];
				} else if (_valid1) {
					valid3 = true;
					passing1 = 1;
				}
				if (!valid3) {
					const err8 = {
						instancePath: instancePath + "/weapon",
						schemaPath: "#/properties/weapon/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing1 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				} else {
					errors = _errs9;
					if (vErrors !== null) {
						if (_errs9) vErrors.length = _errs9;
						else vErrors = null;
					}
				}
			}
		} else {
			const err9 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err9];
			else vErrors.push(err9);
			errors++;
		}
		validate77.errors = vErrors;
		return errors === 0;
	}
	function validate79(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/$defs/ShieldEffect/required",
					keyword: "required",
					params: { missingProperty: "kind" },
					message: "must have required property 'kind'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.remainingTicks === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/$defs/ShieldEffect/required",
					keyword: "required",
					params: { missingProperty: "remainingTicks" },
					message: "must have required property 'remainingTicks'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "kind" || key0 === "remainingTicks")) {
				const err2 = {
					instancePath,
					schemaPath: "#/$defs/ShieldEffect/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("shield" !== data.kind) {
					const err3 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/$defs/ShieldEffect/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "shield" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.remainingTicks !== void 0) {
				let data1 = data.remainingTicks;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err4 = {
						instancePath: instancePath + "/remainingTicks",
						schemaPath: "#/$defs/ShieldEffect/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 300 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/$defs/ShieldEffect/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 300
							},
							message: "must be <= 300"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/$defs/ShieldEffect/properties/remainingTicks/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
				}
			}
		} else {
			const err7 = {
				instancePath,
				schemaPath: "#/$defs/ShieldEffect/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs8 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err8 = {
					instancePath,
					schemaPath: "#/$defs/StopwatchEffect/required",
					keyword: "required",
					params: { missingProperty: "kind" },
					message: "must have required property 'kind'"
				};
				if (vErrors === null) vErrors = [err8];
				else vErrors.push(err8);
				errors++;
			}
			if (data.remainingTicks === void 0) {
				const err9 = {
					instancePath,
					schemaPath: "#/$defs/StopwatchEffect/required",
					keyword: "required",
					params: { missingProperty: "remainingTicks" },
					message: "must have required property 'remainingTicks'"
				};
				if (vErrors === null) vErrors = [err9];
				else vErrors.push(err9);
				errors++;
			}
			for (const key1 in data) if (!(key1 === "kind" || key1 === "remainingTicks")) {
				const err10 = {
					instancePath,
					schemaPath: "#/$defs/StopwatchEffect/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key1 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err10];
				else vErrors.push(err10);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("stopwatch" !== data.kind) {
					const err11 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/$defs/StopwatchEffect/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "stopwatch" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err11];
					else vErrors.push(err11);
					errors++;
				}
			}
			if (data.remainingTicks !== void 0) {
				let data3 = data.remainingTicks;
				if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3) && isFinite(data3))) {
					const err12 = {
						instancePath: instancePath + "/remainingTicks",
						schemaPath: "#/$defs/StopwatchEffect/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				}
				if (typeof data3 == "number" && isFinite(data3)) {
					if (data3 > 180 || isNaN(data3)) {
						const err13 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/$defs/StopwatchEffect/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 180
							},
							message: "must be <= 180"
						};
						if (vErrors === null) vErrors = [err13];
						else vErrors.push(err13);
						errors++;
					}
					if (data3 < 1 || isNaN(data3)) {
						const err14 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/$defs/StopwatchEffect/properties/remainingTicks/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err14];
						else vErrors.push(err14);
						errors++;
					}
				}
			}
		} else {
			const err15 = {
				instancePath,
				schemaPath: "#/$defs/StopwatchEffect/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err15];
			else vErrors.push(err15);
			errors++;
		}
		var _valid0 = _errs8 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else {
			if (_valid0) {
				valid0 = true;
				passing0 = 1;
			}
			const _errs15 = errors;
			if (data && typeof data == "object" && !Array.isArray(data)) {
				if (data.kind === void 0) {
					const err16 = {
						instancePath,
						schemaPath: "#/$defs/InvisibilityCloakEffect/required",
						keyword: "required",
						params: { missingProperty: "kind" },
						message: "must have required property 'kind'"
					};
					if (vErrors === null) vErrors = [err16];
					else vErrors.push(err16);
					errors++;
				}
				if (data.remainingTicks === void 0) {
					const err17 = {
						instancePath,
						schemaPath: "#/$defs/InvisibilityCloakEffect/required",
						keyword: "required",
						params: { missingProperty: "remainingTicks" },
						message: "must have required property 'remainingTicks'"
					};
					if (vErrors === null) vErrors = [err17];
					else vErrors.push(err17);
					errors++;
				}
				for (const key2 in data) if (!(key2 === "kind" || key2 === "remainingTicks")) {
					const err18 = {
						instancePath,
						schemaPath: "#/$defs/InvisibilityCloakEffect/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key2 },
						message: "must NOT have additional properties"
					};
					if (vErrors === null) vErrors = [err18];
					else vErrors.push(err18);
					errors++;
				}
				if (data.kind !== void 0) {
					if ("invisibilityCloak" !== data.kind) {
						const err19 = {
							instancePath: instancePath + "/kind",
							schemaPath: "#/$defs/InvisibilityCloakEffect/properties/kind/const",
							keyword: "const",
							params: { allowedValue: "invisibilityCloak" },
							message: "must be equal to constant"
						};
						if (vErrors === null) vErrors = [err19];
						else vErrors.push(err19);
						errors++;
					}
				}
				if (data.remainingTicks !== void 0) {
					let data5 = data.remainingTicks;
					if (!(typeof data5 == "number" && !(data5 % 1) && !isNaN(data5) && isFinite(data5))) {
						const err20 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/$defs/InvisibilityCloakEffect/properties/remainingTicks/type",
							keyword: "type",
							params: { type: "integer" },
							message: "must be integer"
						};
						if (vErrors === null) vErrors = [err20];
						else vErrors.push(err20);
						errors++;
					}
					if (typeof data5 == "number" && isFinite(data5)) {
						if (data5 > 600 || isNaN(data5)) {
							const err21 = {
								instancePath: instancePath + "/remainingTicks",
								schemaPath: "#/$defs/InvisibilityCloakEffect/properties/remainingTicks/maximum",
								keyword: "maximum",
								params: {
									comparison: "<=",
									limit: 600
								},
								message: "must be <= 600"
							};
							if (vErrors === null) vErrors = [err21];
							else vErrors.push(err21);
							errors++;
						}
						if (data5 < 1 || isNaN(data5)) {
							const err22 = {
								instancePath: instancePath + "/remainingTicks",
								schemaPath: "#/$defs/InvisibilityCloakEffect/properties/remainingTicks/minimum",
								keyword: "minimum",
								params: {
									comparison: ">=",
									limit: 1
								},
								message: "must be >= 1"
							};
							if (vErrors === null) vErrors = [err22];
							else vErrors.push(err22);
							errors++;
						}
					}
				}
			} else {
				const err23 = {
					instancePath,
					schemaPath: "#/$defs/InvisibilityCloakEffect/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err23];
				else vErrors.push(err23);
				errors++;
			}
			var _valid0 = _errs15 === errors;
			if (_valid0 && valid0) {
				valid0 = false;
				passing0 = [passing0, 2];
			} else {
				if (_valid0) {
					valid0 = true;
					passing0 = 2;
				}
				const _errs22 = errors;
				if (data && typeof data == "object" && !Array.isArray(data)) {
					if (data.kind === void 0) {
						const err24 = {
							instancePath,
							schemaPath: "#/$defs/GoldenFlameEffect/required",
							keyword: "required",
							params: { missingProperty: "kind" },
							message: "must have required property 'kind'"
						};
						if (vErrors === null) vErrors = [err24];
						else vErrors.push(err24);
						errors++;
					}
					if (data.remainingTicks === void 0) {
						const err25 = {
							instancePath,
							schemaPath: "#/$defs/GoldenFlameEffect/required",
							keyword: "required",
							params: { missingProperty: "remainingTicks" },
							message: "must have required property 'remainingTicks'"
						};
						if (vErrors === null) vErrors = [err25];
						else vErrors.push(err25);
						errors++;
					}
					for (const key3 in data) if (!(key3 === "kind" || key3 === "remainingTicks")) {
						const err26 = {
							instancePath,
							schemaPath: "#/$defs/GoldenFlameEffect/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key3 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err26];
						else vErrors.push(err26);
						errors++;
					}
					if (data.kind !== void 0) {
						if ("goldenFlame" !== data.kind) {
							const err27 = {
								instancePath: instancePath + "/kind",
								schemaPath: "#/$defs/GoldenFlameEffect/properties/kind/const",
								keyword: "const",
								params: { allowedValue: "goldenFlame" },
								message: "must be equal to constant"
							};
							if (vErrors === null) vErrors = [err27];
							else vErrors.push(err27);
							errors++;
						}
					}
					if (data.remainingTicks !== void 0) {
						let data7 = data.remainingTicks;
						if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7) && isFinite(data7))) {
							const err28 = {
								instancePath: instancePath + "/remainingTicks",
								schemaPath: "#/$defs/GoldenFlameEffect/properties/remainingTicks/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err28];
							else vErrors.push(err28);
							errors++;
						}
						if (typeof data7 == "number" && isFinite(data7)) {
							if (data7 > 600 || isNaN(data7)) {
								const err29 = {
									instancePath: instancePath + "/remainingTicks",
									schemaPath: "#/$defs/GoldenFlameEffect/properties/remainingTicks/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 600
									},
									message: "must be <= 600"
								};
								if (vErrors === null) vErrors = [err29];
								else vErrors.push(err29);
								errors++;
							}
							if (data7 < 1 || isNaN(data7)) {
								const err30 = {
									instancePath: instancePath + "/remainingTicks",
									schemaPath: "#/$defs/GoldenFlameEffect/properties/remainingTicks/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 1
									},
									message: "must be >= 1"
								};
								if (vErrors === null) vErrors = [err30];
								else vErrors.push(err30);
								errors++;
							}
						}
					}
				} else {
					const err31 = {
						instancePath,
						schemaPath: "#/$defs/GoldenFlameEffect/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err31];
					else vErrors.push(err31);
					errors++;
				}
				var _valid0 = _errs22 === errors;
				if (_valid0 && valid0) {
					valid0 = false;
					passing0 = [passing0, 3];
				} else {
					if (_valid0) {
						valid0 = true;
						passing0 = 3;
					}
					const _errs29 = errors;
					if (data && typeof data == "object" && !Array.isArray(data)) {
						if (data.kind === void 0) {
							const err32 = {
								instancePath,
								schemaPath: "#/$defs/ShortFuseEffect/required",
								keyword: "required",
								params: { missingProperty: "kind" },
								message: "must have required property 'kind'"
							};
							if (vErrors === null) vErrors = [err32];
							else vErrors.push(err32);
							errors++;
						}
						if (data.remainingTicks === void 0) {
							const err33 = {
								instancePath,
								schemaPath: "#/$defs/ShortFuseEffect/required",
								keyword: "required",
								params: { missingProperty: "remainingTicks" },
								message: "must have required property 'remainingTicks'"
							};
							if (vErrors === null) vErrors = [err33];
							else vErrors.push(err33);
							errors++;
						}
						for (const key4 in data) if (!(key4 === "kind" || key4 === "remainingTicks")) {
							const err34 = {
								instancePath,
								schemaPath: "#/$defs/ShortFuseEffect/additionalProperties",
								keyword: "additionalProperties",
								params: { additionalProperty: key4 },
								message: "must NOT have additional properties"
							};
							if (vErrors === null) vErrors = [err34];
							else vErrors.push(err34);
							errors++;
						}
						if (data.kind !== void 0) {
							if ("shortFuse" !== data.kind) {
								const err35 = {
									instancePath: instancePath + "/kind",
									schemaPath: "#/$defs/ShortFuseEffect/properties/kind/const",
									keyword: "const",
									params: { allowedValue: "shortFuse" },
									message: "must be equal to constant"
								};
								if (vErrors === null) vErrors = [err35];
								else vErrors.push(err35);
								errors++;
							}
						}
						if (data.remainingTicks !== void 0) {
							let data9 = data.remainingTicks;
							if (!(typeof data9 == "number" && !(data9 % 1) && !isNaN(data9) && isFinite(data9))) {
								const err36 = {
									instancePath: instancePath + "/remainingTicks",
									schemaPath: "#/$defs/ShortFuseEffect/properties/remainingTicks/type",
									keyword: "type",
									params: { type: "integer" },
									message: "must be integer"
								};
								if (vErrors === null) vErrors = [err36];
								else vErrors.push(err36);
								errors++;
							}
							if (typeof data9 == "number" && isFinite(data9)) {
								if (data9 > 600 || isNaN(data9)) {
									const err37 = {
										instancePath: instancePath + "/remainingTicks",
										schemaPath: "#/$defs/ShortFuseEffect/properties/remainingTicks/maximum",
										keyword: "maximum",
										params: {
											comparison: "<=",
											limit: 600
										},
										message: "must be <= 600"
									};
									if (vErrors === null) vErrors = [err37];
									else vErrors.push(err37);
									errors++;
								}
								if (data9 < 1 || isNaN(data9)) {
									const err38 = {
										instancePath: instancePath + "/remainingTicks",
										schemaPath: "#/$defs/ShortFuseEffect/properties/remainingTicks/minimum",
										keyword: "minimum",
										params: {
											comparison: ">=",
											limit: 1
										},
										message: "must be >= 1"
									};
									if (vErrors === null) vErrors = [err38];
									else vErrors.push(err38);
									errors++;
								}
							}
						}
					} else {
						const err39 = {
							instancePath,
							schemaPath: "#/$defs/ShortFuseEffect/type",
							keyword: "type",
							params: { type: "object" },
							message: "must be object"
						};
						if (vErrors === null) vErrors = [err39];
						else vErrors.push(err39);
						errors++;
					}
					var _valid0 = _errs29 === errors;
					if (_valid0 && valid0) {
						valid0 = false;
						passing0 = [passing0, 4];
					} else {
						if (_valid0) {
							valid0 = true;
							passing0 = 4;
						}
						const _errs36 = errors;
						if (data && typeof data == "object" && !Array.isArray(data)) {
							if (data.kind === void 0) {
								const err40 = {
									instancePath,
									schemaPath: "#/$defs/DiarrheaEffect/required",
									keyword: "required",
									params: { missingProperty: "kind" },
									message: "must have required property 'kind'"
								};
								if (vErrors === null) vErrors = [err40];
								else vErrors.push(err40);
								errors++;
							}
							if (data.remainingTicks === void 0) {
								const err41 = {
									instancePath,
									schemaPath: "#/$defs/DiarrheaEffect/required",
									keyword: "required",
									params: { missingProperty: "remainingTicks" },
									message: "must have required property 'remainingTicks'"
								};
								if (vErrors === null) vErrors = [err41];
								else vErrors.push(err41);
								errors++;
							}
							for (const key5 in data) if (!(key5 === "kind" || key5 === "remainingTicks")) {
								const err42 = {
									instancePath,
									schemaPath: "#/$defs/DiarrheaEffect/additionalProperties",
									keyword: "additionalProperties",
									params: { additionalProperty: key5 },
									message: "must NOT have additional properties"
								};
								if (vErrors === null) vErrors = [err42];
								else vErrors.push(err42);
								errors++;
							}
							if (data.kind !== void 0) {
								if ("diarrhea" !== data.kind) {
									const err43 = {
										instancePath: instancePath + "/kind",
										schemaPath: "#/$defs/DiarrheaEffect/properties/kind/const",
										keyword: "const",
										params: { allowedValue: "diarrhea" },
										message: "must be equal to constant"
									};
									if (vErrors === null) vErrors = [err43];
									else vErrors.push(err43);
									errors++;
								}
							}
							if (data.remainingTicks !== void 0) {
								let data11 = data.remainingTicks;
								if (!(typeof data11 == "number" && !(data11 % 1) && !isNaN(data11) && isFinite(data11))) {
									const err44 = {
										instancePath: instancePath + "/remainingTicks",
										schemaPath: "#/$defs/DiarrheaEffect/properties/remainingTicks/type",
										keyword: "type",
										params: { type: "integer" },
										message: "must be integer"
									};
									if (vErrors === null) vErrors = [err44];
									else vErrors.push(err44);
									errors++;
								}
								if (typeof data11 == "number" && isFinite(data11)) {
									if (data11 > 600 || isNaN(data11)) {
										const err45 = {
											instancePath: instancePath + "/remainingTicks",
											schemaPath: "#/$defs/DiarrheaEffect/properties/remainingTicks/maximum",
											keyword: "maximum",
											params: {
												comparison: "<=",
												limit: 600
											},
											message: "must be <= 600"
										};
										if (vErrors === null) vErrors = [err45];
										else vErrors.push(err45);
										errors++;
									}
									if (data11 < 1 || isNaN(data11)) {
										const err46 = {
											instancePath: instancePath + "/remainingTicks",
											schemaPath: "#/$defs/DiarrheaEffect/properties/remainingTicks/minimum",
											keyword: "minimum",
											params: {
												comparison: ">=",
												limit: 1
											},
											message: "must be >= 1"
										};
										if (vErrors === null) vErrors = [err46];
										else vErrors.push(err46);
										errors++;
									}
								}
							}
						} else {
							const err47 = {
								instancePath,
								schemaPath: "#/$defs/DiarrheaEffect/type",
								keyword: "type",
								params: { type: "object" },
								message: "must be object"
							};
							if (vErrors === null) vErrors = [err47];
							else vErrors.push(err47);
							errors++;
						}
						var _valid0 = _errs36 === errors;
						if (_valid0 && valid0) {
							valid0 = false;
							passing0 = [passing0, 5];
						} else {
							if (_valid0) {
								valid0 = true;
								passing0 = 5;
							}
							const _errs43 = errors;
							if (data && typeof data == "object" && !Array.isArray(data)) {
								if (data.kind === void 0) {
									const err48 = {
										instancePath,
										schemaPath: "#/$defs/SlowFootEffect/required",
										keyword: "required",
										params: { missingProperty: "kind" },
										message: "must have required property 'kind'"
									};
									if (vErrors === null) vErrors = [err48];
									else vErrors.push(err48);
									errors++;
								}
								if (data.remainingTicks === void 0) {
									const err49 = {
										instancePath,
										schemaPath: "#/$defs/SlowFootEffect/required",
										keyword: "required",
										params: { missingProperty: "remainingTicks" },
										message: "must have required property 'remainingTicks'"
									};
									if (vErrors === null) vErrors = [err49];
									else vErrors.push(err49);
									errors++;
								}
								for (const key6 in data) if (!(key6 === "kind" || key6 === "remainingTicks")) {
									const err50 = {
										instancePath,
										schemaPath: "#/$defs/SlowFootEffect/additionalProperties",
										keyword: "additionalProperties",
										params: { additionalProperty: key6 },
										message: "must NOT have additional properties"
									};
									if (vErrors === null) vErrors = [err50];
									else vErrors.push(err50);
									errors++;
								}
								if (data.kind !== void 0) {
									if ("slowFoot" !== data.kind) {
										const err51 = {
											instancePath: instancePath + "/kind",
											schemaPath: "#/$defs/SlowFootEffect/properties/kind/const",
											keyword: "const",
											params: { allowedValue: "slowFoot" },
											message: "must be equal to constant"
										};
										if (vErrors === null) vErrors = [err51];
										else vErrors.push(err51);
										errors++;
									}
								}
								if (data.remainingTicks !== void 0) {
									let data13 = data.remainingTicks;
									if (!(typeof data13 == "number" && !(data13 % 1) && !isNaN(data13) && isFinite(data13))) {
										const err52 = {
											instancePath: instancePath + "/remainingTicks",
											schemaPath: "#/$defs/SlowFootEffect/properties/remainingTicks/type",
											keyword: "type",
											params: { type: "integer" },
											message: "must be integer"
										};
										if (vErrors === null) vErrors = [err52];
										else vErrors.push(err52);
										errors++;
									}
									if (typeof data13 == "number" && isFinite(data13)) {
										if (data13 > 600 || isNaN(data13)) {
											const err53 = {
												instancePath: instancePath + "/remainingTicks",
												schemaPath: "#/$defs/SlowFootEffect/properties/remainingTicks/maximum",
												keyword: "maximum",
												params: {
													comparison: "<=",
													limit: 600
												},
												message: "must be <= 600"
											};
											if (vErrors === null) vErrors = [err53];
											else vErrors.push(err53);
											errors++;
										}
										if (data13 < 1 || isNaN(data13)) {
											const err54 = {
												instancePath: instancePath + "/remainingTicks",
												schemaPath: "#/$defs/SlowFootEffect/properties/remainingTicks/minimum",
												keyword: "minimum",
												params: {
													comparison: ">=",
													limit: 1
												},
												message: "must be >= 1"
											};
											if (vErrors === null) vErrors = [err54];
											else vErrors.push(err54);
											errors++;
										}
									}
								}
							} else {
								const err55 = {
									instancePath,
									schemaPath: "#/$defs/SlowFootEffect/type",
									keyword: "type",
									params: { type: "object" },
									message: "must be object"
								};
								if (vErrors === null) vErrors = [err55];
								else vErrors.push(err55);
								errors++;
							}
							var _valid0 = _errs43 === errors;
							if (_valid0 && valid0) {
								valid0 = false;
								passing0 = [passing0, 6];
							} else {
								if (_valid0) {
									valid0 = true;
									passing0 = 6;
								}
								const _errs50 = errors;
								if (data && typeof data == "object" && !Array.isArray(data)) {
									if (data.kind === void 0) {
										const err56 = {
											instancePath,
											schemaPath: "#/$defs/ShortFlameEffect/required",
											keyword: "required",
											params: { missingProperty: "kind" },
											message: "must have required property 'kind'"
										};
										if (vErrors === null) vErrors = [err56];
										else vErrors.push(err56);
										errors++;
									}
									if (data.remainingTicks === void 0) {
										const err57 = {
											instancePath,
											schemaPath: "#/$defs/ShortFlameEffect/required",
											keyword: "required",
											params: { missingProperty: "remainingTicks" },
											message: "must have required property 'remainingTicks'"
										};
										if (vErrors === null) vErrors = [err57];
										else vErrors.push(err57);
										errors++;
									}
									for (const key7 in data) if (!(key7 === "kind" || key7 === "remainingTicks")) {
										const err58 = {
											instancePath,
											schemaPath: "#/$defs/ShortFlameEffect/additionalProperties",
											keyword: "additionalProperties",
											params: { additionalProperty: key7 },
											message: "must NOT have additional properties"
										};
										if (vErrors === null) vErrors = [err58];
										else vErrors.push(err58);
										errors++;
									}
									if (data.kind !== void 0) {
										if ("shortFlame" !== data.kind) {
											const err59 = {
												instancePath: instancePath + "/kind",
												schemaPath: "#/$defs/ShortFlameEffect/properties/kind/const",
												keyword: "const",
												params: { allowedValue: "shortFlame" },
												message: "must be equal to constant"
											};
											if (vErrors === null) vErrors = [err59];
											else vErrors.push(err59);
											errors++;
										}
									}
									if (data.remainingTicks !== void 0) {
										let data15 = data.remainingTicks;
										if (!(typeof data15 == "number" && !(data15 % 1) && !isNaN(data15) && isFinite(data15))) {
											const err60 = {
												instancePath: instancePath + "/remainingTicks",
												schemaPath: "#/$defs/ShortFlameEffect/properties/remainingTicks/type",
												keyword: "type",
												params: { type: "integer" },
												message: "must be integer"
											};
											if (vErrors === null) vErrors = [err60];
											else vErrors.push(err60);
											errors++;
										}
										if (typeof data15 == "number" && isFinite(data15)) {
											if (data15 > 600 || isNaN(data15)) {
												const err61 = {
													instancePath: instancePath + "/remainingTicks",
													schemaPath: "#/$defs/ShortFlameEffect/properties/remainingTicks/maximum",
													keyword: "maximum",
													params: {
														comparison: "<=",
														limit: 600
													},
													message: "must be <= 600"
												};
												if (vErrors === null) vErrors = [err61];
												else vErrors.push(err61);
												errors++;
											}
											if (data15 < 1 || isNaN(data15)) {
												const err62 = {
													instancePath: instancePath + "/remainingTicks",
													schemaPath: "#/$defs/ShortFlameEffect/properties/remainingTicks/minimum",
													keyword: "minimum",
													params: {
														comparison: ">=",
														limit: 1
													},
													message: "must be >= 1"
												};
												if (vErrors === null) vErrors = [err62];
												else vErrors.push(err62);
												errors++;
											}
										}
									}
								} else {
									const err63 = {
										instancePath,
										schemaPath: "#/$defs/ShortFlameEffect/type",
										keyword: "type",
										params: { type: "object" },
										message: "must be object"
									};
									if (vErrors === null) vErrors = [err63];
									else vErrors.push(err63);
									errors++;
								}
								var _valid0 = _errs50 === errors;
								if (_valid0 && valid0) {
									valid0 = false;
									passing0 = [passing0, 7];
								} else {
									if (_valid0) {
										valid0 = true;
										passing0 = 7;
									}
									const _errs57 = errors;
									if (data && typeof data == "object" && !Array.isArray(data)) {
										if (data.kind === void 0) {
											const err64 = {
												instancePath,
												schemaPath: "#/$defs/LongFuseEffect/required",
												keyword: "required",
												params: { missingProperty: "kind" },
												message: "must have required property 'kind'"
											};
											if (vErrors === null) vErrors = [err64];
											else vErrors.push(err64);
											errors++;
										}
										if (data.remainingTicks === void 0) {
											const err65 = {
												instancePath,
												schemaPath: "#/$defs/LongFuseEffect/required",
												keyword: "required",
												params: { missingProperty: "remainingTicks" },
												message: "must have required property 'remainingTicks'"
											};
											if (vErrors === null) vErrors = [err65];
											else vErrors.push(err65);
											errors++;
										}
										for (const key8 in data) if (!(key8 === "kind" || key8 === "remainingTicks")) {
											const err66 = {
												instancePath,
												schemaPath: "#/$defs/LongFuseEffect/additionalProperties",
												keyword: "additionalProperties",
												params: { additionalProperty: key8 },
												message: "must NOT have additional properties"
											};
											if (vErrors === null) vErrors = [err66];
											else vErrors.push(err66);
											errors++;
										}
										if (data.kind !== void 0) {
											if ("longFuse" !== data.kind) {
												const err67 = {
													instancePath: instancePath + "/kind",
													schemaPath: "#/$defs/LongFuseEffect/properties/kind/const",
													keyword: "const",
													params: { allowedValue: "longFuse" },
													message: "must be equal to constant"
												};
												if (vErrors === null) vErrors = [err67];
												else vErrors.push(err67);
												errors++;
											}
										}
										if (data.remainingTicks !== void 0) {
											let data17 = data.remainingTicks;
											if (!(typeof data17 == "number" && !(data17 % 1) && !isNaN(data17) && isFinite(data17))) {
												const err68 = {
													instancePath: instancePath + "/remainingTicks",
													schemaPath: "#/$defs/LongFuseEffect/properties/remainingTicks/type",
													keyword: "type",
													params: { type: "integer" },
													message: "must be integer"
												};
												if (vErrors === null) vErrors = [err68];
												else vErrors.push(err68);
												errors++;
											}
											if (typeof data17 == "number" && isFinite(data17)) {
												if (data17 > 600 || isNaN(data17)) {
													const err69 = {
														instancePath: instancePath + "/remainingTicks",
														schemaPath: "#/$defs/LongFuseEffect/properties/remainingTicks/maximum",
														keyword: "maximum",
														params: {
															comparison: "<=",
															limit: 600
														},
														message: "must be <= 600"
													};
													if (vErrors === null) vErrors = [err69];
													else vErrors.push(err69);
													errors++;
												}
												if (data17 < 1 || isNaN(data17)) {
													const err70 = {
														instancePath: instancePath + "/remainingTicks",
														schemaPath: "#/$defs/LongFuseEffect/properties/remainingTicks/minimum",
														keyword: "minimum",
														params: {
															comparison: ">=",
															limit: 1
														},
														message: "must be >= 1"
													};
													if (vErrors === null) vErrors = [err70];
													else vErrors.push(err70);
													errors++;
												}
											}
										}
									} else {
										const err71 = {
											instancePath,
											schemaPath: "#/$defs/LongFuseEffect/type",
											keyword: "type",
											params: { type: "object" },
											message: "must be object"
										};
										if (vErrors === null) vErrors = [err71];
										else vErrors.push(err71);
										errors++;
									}
									var _valid0 = _errs57 === errors;
									if (_valid0 && valid0) {
										valid0 = false;
										passing0 = [passing0, 8];
									} else {
										if (_valid0) {
											valid0 = true;
											passing0 = 8;
										}
										const _errs64 = errors;
										if (data && typeof data == "object" && !Array.isArray(data)) {
											if (data.kind === void 0) {
												const err72 = {
													instancePath,
													schemaPath: "#/$defs/ReverseControlsEffect/required",
													keyword: "required",
													params: { missingProperty: "kind" },
													message: "must have required property 'kind'"
												};
												if (vErrors === null) vErrors = [err72];
												else vErrors.push(err72);
												errors++;
											}
											if (data.remainingTicks === void 0) {
												const err73 = {
													instancePath,
													schemaPath: "#/$defs/ReverseControlsEffect/required",
													keyword: "required",
													params: { missingProperty: "remainingTicks" },
													message: "must have required property 'remainingTicks'"
												};
												if (vErrors === null) vErrors = [err73];
												else vErrors.push(err73);
												errors++;
											}
											for (const key9 in data) if (!(key9 === "kind" || key9 === "remainingTicks")) {
												const err74 = {
													instancePath,
													schemaPath: "#/$defs/ReverseControlsEffect/additionalProperties",
													keyword: "additionalProperties",
													params: { additionalProperty: key9 },
													message: "must NOT have additional properties"
												};
												if (vErrors === null) vErrors = [err74];
												else vErrors.push(err74);
												errors++;
											}
											if (data.kind !== void 0) {
												if ("reverseControls" !== data.kind) {
													const err75 = {
														instancePath: instancePath + "/kind",
														schemaPath: "#/$defs/ReverseControlsEffect/properties/kind/const",
														keyword: "const",
														params: { allowedValue: "reverseControls" },
														message: "must be equal to constant"
													};
													if (vErrors === null) vErrors = [err75];
													else vErrors.push(err75);
													errors++;
												}
											}
											if (data.remainingTicks !== void 0) {
												let data19 = data.remainingTicks;
												if (!(typeof data19 == "number" && !(data19 % 1) && !isNaN(data19) && isFinite(data19))) {
													const err76 = {
														instancePath: instancePath + "/remainingTicks",
														schemaPath: "#/$defs/ReverseControlsEffect/properties/remainingTicks/type",
														keyword: "type",
														params: { type: "integer" },
														message: "must be integer"
													};
													if (vErrors === null) vErrors = [err76];
													else vErrors.push(err76);
													errors++;
												}
												if (typeof data19 == "number" && isFinite(data19)) {
													if (data19 > 600 || isNaN(data19)) {
														const err77 = {
															instancePath: instancePath + "/remainingTicks",
															schemaPath: "#/$defs/ReverseControlsEffect/properties/remainingTicks/maximum",
															keyword: "maximum",
															params: {
																comparison: "<=",
																limit: 600
															},
															message: "must be <= 600"
														};
														if (vErrors === null) vErrors = [err77];
														else vErrors.push(err77);
														errors++;
													}
													if (data19 < 1 || isNaN(data19)) {
														const err78 = {
															instancePath: instancePath + "/remainingTicks",
															schemaPath: "#/$defs/ReverseControlsEffect/properties/remainingTicks/minimum",
															keyword: "minimum",
															params: {
																comparison: ">=",
																limit: 1
															},
															message: "must be >= 1"
														};
														if (vErrors === null) vErrors = [err78];
														else vErrors.push(err78);
														errors++;
													}
												}
											}
										} else {
											const err79 = {
												instancePath,
												schemaPath: "#/$defs/ReverseControlsEffect/type",
												keyword: "type",
												params: { type: "object" },
												message: "must be object"
											};
											if (vErrors === null) vErrors = [err79];
											else vErrors.push(err79);
											errors++;
										}
										var _valid0 = _errs64 === errors;
										if (_valid0 && valid0) {
											valid0 = false;
											passing0 = [passing0, 9];
										} else {
											if (_valid0) {
												valid0 = true;
												passing0 = 9;
											}
											const _errs71 = errors;
											if (data && typeof data == "object" && !Array.isArray(data)) {
												if (data.kind === void 0) {
													const err80 = {
														instancePath,
														schemaPath: "#/$defs/InvisibleBombsEffect/required",
														keyword: "required",
														params: { missingProperty: "kind" },
														message: "must have required property 'kind'"
													};
													if (vErrors === null) vErrors = [err80];
													else vErrors.push(err80);
													errors++;
												}
												if (data.remainingTicks === void 0) {
													const err81 = {
														instancePath,
														schemaPath: "#/$defs/InvisibleBombsEffect/required",
														keyword: "required",
														params: { missingProperty: "remainingTicks" },
														message: "must have required property 'remainingTicks'"
													};
													if (vErrors === null) vErrors = [err81];
													else vErrors.push(err81);
													errors++;
												}
												for (const key10 in data) if (!(key10 === "kind" || key10 === "remainingTicks")) {
													const err82 = {
														instancePath,
														schemaPath: "#/$defs/InvisibleBombsEffect/additionalProperties",
														keyword: "additionalProperties",
														params: { additionalProperty: key10 },
														message: "must NOT have additional properties"
													};
													if (vErrors === null) vErrors = [err82];
													else vErrors.push(err82);
													errors++;
												}
												if (data.kind !== void 0) {
													if ("invisibleBombs" !== data.kind) {
														const err83 = {
															instancePath: instancePath + "/kind",
															schemaPath: "#/$defs/InvisibleBombsEffect/properties/kind/const",
															keyword: "const",
															params: { allowedValue: "invisibleBombs" },
															message: "must be equal to constant"
														};
														if (vErrors === null) vErrors = [err83];
														else vErrors.push(err83);
														errors++;
													}
												}
												if (data.remainingTicks !== void 0) {
													let data21 = data.remainingTicks;
													if (!(typeof data21 == "number" && !(data21 % 1) && !isNaN(data21) && isFinite(data21))) {
														const err84 = {
															instancePath: instancePath + "/remainingTicks",
															schemaPath: "#/$defs/InvisibleBombsEffect/properties/remainingTicks/type",
															keyword: "type",
															params: { type: "integer" },
															message: "must be integer"
														};
														if (vErrors === null) vErrors = [err84];
														else vErrors.push(err84);
														errors++;
													}
													if (typeof data21 == "number" && isFinite(data21)) {
														if (data21 > 600 || isNaN(data21)) {
															const err85 = {
																instancePath: instancePath + "/remainingTicks",
																schemaPath: "#/$defs/InvisibleBombsEffect/properties/remainingTicks/maximum",
																keyword: "maximum",
																params: {
																	comparison: "<=",
																	limit: 600
																},
																message: "must be <= 600"
															};
															if (vErrors === null) vErrors = [err85];
															else vErrors.push(err85);
															errors++;
														}
														if (data21 < 1 || isNaN(data21)) {
															const err86 = {
																instancePath: instancePath + "/remainingTicks",
																schemaPath: "#/$defs/InvisibleBombsEffect/properties/remainingTicks/minimum",
																keyword: "minimum",
																params: {
																	comparison: ">=",
																	limit: 1
																},
																message: "must be >= 1"
															};
															if (vErrors === null) vErrors = [err86];
															else vErrors.push(err86);
															errors++;
														}
													}
												}
											} else {
												const err87 = {
													instancePath,
													schemaPath: "#/$defs/InvisibleBombsEffect/type",
													keyword: "type",
													params: { type: "object" },
													message: "must be object"
												};
												if (vErrors === null) vErrors = [err87];
												else vErrors.push(err87);
												errors++;
											}
											var _valid0 = _errs71 === errors;
											if (_valid0 && valid0) {
												valid0 = false;
												passing0 = [passing0, 10];
											} else if (_valid0) {
												valid0 = true;
												passing0 = 10;
											}
										}
									}
								}
							}
						}
					}
				}
			}
		}
		if (!valid0) {
			const err88 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err88];
			else vErrors.push(err88);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate79.errors = vErrors;
		return errors === 0;
	}
	function validate76(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.id === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "id" },
					message: "must have required property 'id'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.seat === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "seat" },
					message: "must have required property 'seat'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.position === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "position" },
					message: "must have required property 'position'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.heading === void 0) {
				const err4 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "heading" },
					message: "must have required property 'heading'"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.speedLevel === void 0) {
				const err5 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "speedLevel" },
					message: "must have required property 'speedLevel'"
				};
				if (vErrors === null) vErrors = [err5];
				else vErrors.push(err5);
				errors++;
			}
			if (data.alive === void 0) {
				const err6 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "alive" },
					message: "must have required property 'alive'"
				};
				if (vErrors === null) vErrors = [err6];
				else vErrors.push(err6);
				errors++;
			}
			if (data.bombCapacity === void 0) {
				const err7 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "bombCapacity" },
					message: "must have required property 'bombCapacity'"
				};
				if (vErrors === null) vErrors = [err7];
				else vErrors.push(err7);
				errors++;
			}
			if (data.bombRange === void 0) {
				const err8 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "bombRange" },
					message: "must have required property 'bombRange'"
				};
				if (vErrors === null) vErrors = [err8];
				else vErrors.push(err8);
				errors++;
			}
			if (data.liveBombCount === void 0) {
				const err9 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "liveBombCount" },
					message: "must have required property 'liveBombCount'"
				};
				if (vErrors === null) vErrors = [err9];
				else vErrors.push(err9);
				errors++;
			}
			if (data.inventory === void 0) {
				const err10 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "inventory" },
					message: "must have required property 'inventory'"
				};
				if (vErrors === null) vErrors = [err10];
				else vErrors.push(err10);
				errors++;
			}
			if (data.effects === void 0) {
				const err11 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "effects" },
					message: "must have required property 'effects'"
				};
				if (vErrors === null) vErrors = [err11];
				else vErrors.push(err11);
				errors++;
			}
			if (data.damageGraceRemainingTicks === void 0) {
				const err12 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "damageGraceRemainingTicks" },
					message: "must have required property 'damageGraceRemainingTicks'"
				};
				if (vErrors === null) vErrors = [err12];
				else vErrors.push(err12);
				errors++;
			}
			if (data.voluntaryActionsFrozen === void 0) {
				const err13 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "voluntaryActionsFrozen" },
					message: "must have required property 'voluntaryActionsFrozen'"
				};
				if (vErrors === null) vErrors = [err13];
				else vErrors.push(err13);
				errors++;
			}
			if (data.lastTeleportTick === void 0) {
				const err14 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "lastTeleportTick" },
					message: "must have required property 'lastTeleportTick'"
				};
				if (vErrors === null) vErrors = [err14];
				else vErrors.push(err14);
				errors++;
			}
			if (data.stormSlowRemainingTicks === void 0) {
				const err15 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "stormSlowRemainingTicks" },
					message: "must have required property 'stormSlowRemainingTicks'"
				};
				if (vErrors === null) vErrors = [err15];
				else vErrors.push(err15);
				errors++;
			}
			for (const key0 in data) if (!func2.call(schema105.properties, key0)) {
				const err16 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err16];
				else vErrors.push(err16);
				errors++;
			}
			if (data.type !== void 0) {
				if ("visible" !== data.type) {
					const err17 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "visible" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err17];
					else vErrors.push(err17);
					errors++;
				}
			}
			if (data.id !== void 0) {
				let data1 = data.id;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err18 = {
						instancePath: instancePath + "/id",
						schemaPath: "#/$defs/EntityId/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err18];
					else vErrors.push(err18);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 4294967295 || isNaN(data1)) {
						const err19 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 4294967295
							},
							message: "must be <= 4294967295"
						};
						if (vErrors === null) vErrors = [err19];
						else vErrors.push(err19);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err20 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err20];
						else vErrors.push(err20);
						errors++;
					}
				}
			}
			if (data.seat !== void 0) {
				let data2 = data.seat;
				if (!(data2 === "one" || data2 === "two" || data2 === "three" || data2 === "four")) {
					const err21 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/Seat/enum",
						keyword: "enum",
						params: { allowedValues: schema30.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err21];
					else vErrors.push(err21);
					errors++;
				}
			}
			if (data.position !== void 0) {
				let data3 = data.position;
				if (data3 && typeof data3 == "object" && !Array.isArray(data3)) {
					if (data3.x === void 0) {
						const err22 = {
							instancePath: instancePath + "/position",
							schemaPath: "#/$defs/Position/required",
							keyword: "required",
							params: { missingProperty: "x" },
							message: "must have required property 'x'"
						};
						if (vErrors === null) vErrors = [err22];
						else vErrors.push(err22);
						errors++;
					}
					if (data3.y === void 0) {
						const err23 = {
							instancePath: instancePath + "/position",
							schemaPath: "#/$defs/Position/required",
							keyword: "required",
							params: { missingProperty: "y" },
							message: "must have required property 'y'"
						};
						if (vErrors === null) vErrors = [err23];
						else vErrors.push(err23);
						errors++;
					}
					for (const key1 in data3) if (!(key1 === "x" || key1 === "y")) {
						const err24 = {
							instancePath: instancePath + "/position",
							schemaPath: "#/$defs/Position/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err24];
						else vErrors.push(err24);
						errors++;
					}
					if (data3.x !== void 0) {
						let data4 = data3.x;
						if (!(typeof data4 == "number" && !(data4 % 1) && !isNaN(data4) && isFinite(data4))) {
							const err25 = {
								instancePath: instancePath + "/position/x",
								schemaPath: "#/$defs/Position/properties/x/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err25];
							else vErrors.push(err25);
							errors++;
						}
						if (typeof data4 == "number" && isFinite(data4)) {
							if (data4 > 15359 || isNaN(data4)) {
								const err26 = {
									instancePath: instancePath + "/position/x",
									schemaPath: "#/$defs/Position/properties/x/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 15359
									},
									message: "must be <= 15359"
								};
								if (vErrors === null) vErrors = [err26];
								else vErrors.push(err26);
								errors++;
							}
							if (data4 < 0 || isNaN(data4)) {
								const err27 = {
									instancePath: instancePath + "/position/x",
									schemaPath: "#/$defs/Position/properties/x/minimum",
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
					if (data3.y !== void 0) {
						let data5 = data3.y;
						if (!(typeof data5 == "number" && !(data5 % 1) && !isNaN(data5) && isFinite(data5))) {
							const err28 = {
								instancePath: instancePath + "/position/y",
								schemaPath: "#/$defs/Position/properties/y/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err28];
							else vErrors.push(err28);
							errors++;
						}
						if (typeof data5 == "number" && isFinite(data5)) {
							if (data5 > 13311 || isNaN(data5)) {
								const err29 = {
									instancePath: instancePath + "/position/y",
									schemaPath: "#/$defs/Position/properties/y/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 13311
									},
									message: "must be <= 13311"
								};
								if (vErrors === null) vErrors = [err29];
								else vErrors.push(err29);
								errors++;
							}
							if (data5 < 0 || isNaN(data5)) {
								const err30 = {
									instancePath: instancePath + "/position/y",
									schemaPath: "#/$defs/Position/properties/y/minimum",
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
						instancePath: instancePath + "/position",
						schemaPath: "#/$defs/Position/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err31];
					else vErrors.push(err31);
					errors++;
				}
			}
			if (data.heading !== void 0) {
				let data6 = data.heading;
				if (!(data6 === "north" || data6 === "east" || data6 === "south" || data6 === "west")) {
					const err32 = {
						instancePath: instancePath + "/heading",
						schemaPath: "#/$defs/Direction/enum",
						keyword: "enum",
						params: { allowedValues: schema109.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err32];
					else vErrors.push(err32);
					errors++;
				}
			}
			if (data.speedLevel !== void 0) {
				let data7 = data.speedLevel;
				if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7) && isFinite(data7))) {
					const err33 = {
						instancePath: instancePath + "/speedLevel",
						schemaPath: "#/properties/speedLevel/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err33];
					else vErrors.push(err33);
					errors++;
				}
				if (typeof data7 == "number" && isFinite(data7)) {
					if (data7 > 4 || isNaN(data7)) {
						const err34 = {
							instancePath: instancePath + "/speedLevel",
							schemaPath: "#/properties/speedLevel/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 4
							},
							message: "must be <= 4"
						};
						if (vErrors === null) vErrors = [err34];
						else vErrors.push(err34);
						errors++;
					}
					if (data7 < 0 || isNaN(data7)) {
						const err35 = {
							instancePath: instancePath + "/speedLevel",
							schemaPath: "#/properties/speedLevel/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err35];
						else vErrors.push(err35);
						errors++;
					}
				}
			}
			if (data.alive !== void 0) {
				if (typeof data.alive !== "boolean") {
					const err36 = {
						instancePath: instancePath + "/alive",
						schemaPath: "#/properties/alive/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err36];
					else vErrors.push(err36);
					errors++;
				}
			}
			if (data.bombCapacity !== void 0) {
				let data9 = data.bombCapacity;
				if (!(typeof data9 == "number" && !(data9 % 1) && !isNaN(data9) && isFinite(data9))) {
					const err37 = {
						instancePath: instancePath + "/bombCapacity",
						schemaPath: "#/properties/bombCapacity/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err37];
					else vErrors.push(err37);
					errors++;
				}
				if (typeof data9 == "number" && isFinite(data9)) {
					if (data9 > 8 || isNaN(data9)) {
						const err38 = {
							instancePath: instancePath + "/bombCapacity",
							schemaPath: "#/properties/bombCapacity/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 8
							},
							message: "must be <= 8"
						};
						if (vErrors === null) vErrors = [err38];
						else vErrors.push(err38);
						errors++;
					}
					if (data9 < 1 || isNaN(data9)) {
						const err39 = {
							instancePath: instancePath + "/bombCapacity",
							schemaPath: "#/properties/bombCapacity/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err39];
						else vErrors.push(err39);
						errors++;
					}
				}
			}
			if (data.bombRange !== void 0) {
				let data10 = data.bombRange;
				if (!(typeof data10 == "number" && !(data10 % 1) && !isNaN(data10) && isFinite(data10))) {
					const err40 = {
						instancePath: instancePath + "/bombRange",
						schemaPath: "#/properties/bombRange/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err40];
					else vErrors.push(err40);
					errors++;
				}
				if (typeof data10 == "number" && isFinite(data10)) {
					if (data10 > 8 || isNaN(data10)) {
						const err41 = {
							instancePath: instancePath + "/bombRange",
							schemaPath: "#/properties/bombRange/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 8
							},
							message: "must be <= 8"
						};
						if (vErrors === null) vErrors = [err41];
						else vErrors.push(err41);
						errors++;
					}
					if (data10 < 1 || isNaN(data10)) {
						const err42 = {
							instancePath: instancePath + "/bombRange",
							schemaPath: "#/properties/bombRange/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err42];
						else vErrors.push(err42);
						errors++;
					}
				}
			}
			if (data.liveBombCount !== void 0) {
				let data11 = data.liveBombCount;
				if (!(typeof data11 == "number" && !(data11 % 1) && !isNaN(data11) && isFinite(data11))) {
					const err43 = {
						instancePath: instancePath + "/liveBombCount",
						schemaPath: "#/properties/liveBombCount/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err43];
					else vErrors.push(err43);
					errors++;
				}
				if (typeof data11 == "number" && isFinite(data11)) {
					if (data11 > 8 || isNaN(data11)) {
						const err44 = {
							instancePath: instancePath + "/liveBombCount",
							schemaPath: "#/properties/liveBombCount/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 8
							},
							message: "must be <= 8"
						};
						if (vErrors === null) vErrors = [err44];
						else vErrors.push(err44);
						errors++;
					}
					if (data11 < 0 || isNaN(data11)) {
						const err45 = {
							instancePath: instancePath + "/liveBombCount",
							schemaPath: "#/properties/liveBombCount/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err45];
						else vErrors.push(err45);
						errors++;
					}
				}
			}
			if (data.inventory !== void 0) {
				if (!validate77(data.inventory, {
					instancePath: instancePath + "/inventory",
					parentData: data,
					parentDataProperty: "inventory",
					rootData
				})) {
					vErrors = vErrors === null ? validate77.errors : vErrors.concat(validate77.errors);
					errors = vErrors.length;
				}
			}
			if (data.effects !== void 0) {
				let data13 = data.effects;
				if (Array.isArray(data13)) {
					if (data13.length > 16) {
						const err46 = {
							instancePath: instancePath + "/effects",
							schemaPath: "#/properties/effects/maxItems",
							keyword: "maxItems",
							params: { limit: 16 },
							message: "must NOT have more than 16 items"
						};
						if (vErrors === null) vErrors = [err46];
						else vErrors.push(err46);
						errors++;
					}
					if (data13.length < 0) {
						const err47 = {
							instancePath: instancePath + "/effects",
							schemaPath: "#/properties/effects/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err47];
						else vErrors.push(err47);
						errors++;
					}
					const len0 = data13.length;
					for (let i0 = 0; i0 < len0; i0++) if (!validate79(data13[i0], {
						instancePath: instancePath + "/effects/" + i0,
						parentData: data13,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate79.errors : vErrors.concat(validate79.errors);
						errors = vErrors.length;
					}
				} else {
					const err48 = {
						instancePath: instancePath + "/effects",
						schemaPath: "#/properties/effects/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err48];
					else vErrors.push(err48);
					errors++;
				}
			}
			if (data.damageGraceRemainingTicks !== void 0) {
				let data15 = data.damageGraceRemainingTicks;
				if (!(typeof data15 == "number" && !(data15 % 1) && !isNaN(data15) && isFinite(data15))) {
					const err49 = {
						instancePath: instancePath + "/damageGraceRemainingTicks",
						schemaPath: "#/properties/damageGraceRemainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err49];
					else vErrors.push(err49);
					errors++;
				}
				if (typeof data15 == "number" && isFinite(data15)) {
					if (data15 > 60 || isNaN(data15)) {
						const err50 = {
							instancePath: instancePath + "/damageGraceRemainingTicks",
							schemaPath: "#/properties/damageGraceRemainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 60
							},
							message: "must be <= 60"
						};
						if (vErrors === null) vErrors = [err50];
						else vErrors.push(err50);
						errors++;
					}
					if (data15 < 0 || isNaN(data15)) {
						const err51 = {
							instancePath: instancePath + "/damageGraceRemainingTicks",
							schemaPath: "#/properties/damageGraceRemainingTicks/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err51];
						else vErrors.push(err51);
						errors++;
					}
				}
			}
			if (data.voluntaryActionsFrozen !== void 0) {
				if (typeof data.voluntaryActionsFrozen !== "boolean") {
					const err52 = {
						instancePath: instancePath + "/voluntaryActionsFrozen",
						schemaPath: "#/properties/voluntaryActionsFrozen/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err52];
					else vErrors.push(err52);
					errors++;
				}
			}
			if (data.lastTeleportTick !== void 0) {
				let data17 = data.lastTeleportTick;
				const _errs37 = errors;
				let valid8 = false;
				let passing0 = null;
				const _errs38 = errors;
				if (!(typeof data17 == "number" && !(data17 % 1) && !isNaN(data17) && isFinite(data17))) {
					const err53 = {
						instancePath: instancePath + "/lastTeleportTick",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err53];
					else vErrors.push(err53);
					errors++;
				}
				if (typeof data17 == "number" && isFinite(data17)) {
					if (data17 > 9007199254740991 || isNaN(data17)) {
						const err54 = {
							instancePath: instancePath + "/lastTeleportTick",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err54];
						else vErrors.push(err54);
						errors++;
					}
					if (data17 < 0 || isNaN(data17)) {
						const err55 = {
							instancePath: instancePath + "/lastTeleportTick",
							schemaPath: "#/$defs/Tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err55];
						else vErrors.push(err55);
						errors++;
					}
				}
				var _valid0 = _errs38 === errors;
				if (_valid0) {
					valid8 = true;
					passing0 = 0;
				}
				const _errs41 = errors;
				if (data17 !== null) {
					const err56 = {
						instancePath: instancePath + "/lastTeleportTick",
						schemaPath: "#/properties/lastTeleportTick/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err56];
					else vErrors.push(err56);
					errors++;
				}
				var _valid0 = _errs41 === errors;
				if (_valid0 && valid8) {
					valid8 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid8 = true;
					passing0 = 1;
				}
				if (!valid8) {
					const err57 = {
						instancePath: instancePath + "/lastTeleportTick",
						schemaPath: "#/properties/lastTeleportTick/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err57];
					else vErrors.push(err57);
					errors++;
				} else {
					errors = _errs37;
					if (vErrors !== null) {
						if (_errs37) vErrors.length = _errs37;
						else vErrors = null;
					}
				}
			}
			if (data.stormSlowRemainingTicks !== void 0) {
				let data18 = data.stormSlowRemainingTicks;
				if (!(typeof data18 == "number" && !(data18 % 1) && !isNaN(data18) && isFinite(data18))) {
					const err58 = {
						instancePath: instancePath + "/stormSlowRemainingTicks",
						schemaPath: "#/properties/stormSlowRemainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err58];
					else vErrors.push(err58);
					errors++;
				}
				if (typeof data18 == "number" && isFinite(data18)) {
					if (data18 > 180 || isNaN(data18)) {
						const err59 = {
							instancePath: instancePath + "/stormSlowRemainingTicks",
							schemaPath: "#/properties/stormSlowRemainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 180
							},
							message: "must be <= 180"
						};
						if (vErrors === null) vErrors = [err59];
						else vErrors.push(err59);
						errors++;
					}
					if (data18 < 0 || isNaN(data18)) {
						const err60 = {
							instancePath: instancePath + "/stormSlowRemainingTicks",
							schemaPath: "#/properties/stormSlowRemainingTicks/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err60];
						else vErrors.push(err60);
						errors++;
					}
				}
			}
		} else {
			const err61 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err61];
			else vErrors.push(err61);
			errors++;
		}
		validate76.errors = vErrors;
		return errors === 0;
	}
	function validate82(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.id === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "id" },
					message: "must have required property 'id'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.seat === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "seat" },
					message: "must have required property 'seat'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.alive === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "alive" },
					message: "must have required property 'alive'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "id" || key0 === "seat" || key0 === "alive")) {
				const err4 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.type !== void 0) {
				if ("hidden" !== data.type) {
					const err5 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "hidden" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.id !== void 0) {
				let data1 = data.id;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err6 = {
						instancePath: instancePath + "/id",
						schemaPath: "#/$defs/EntityId/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 4294967295 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 4294967295
							},
							message: "must be <= 4294967295"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err8 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
				}
			}
			if (data.seat !== void 0) {
				let data2 = data.seat;
				if (!(data2 === "one" || data2 === "two" || data2 === "three" || data2 === "four")) {
					const err9 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/Seat/enum",
						keyword: "enum",
						params: { allowedValues: schema30.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
			}
			if (data.alive !== void 0) {
				if (typeof data.alive !== "boolean") {
					const err10 = {
						instancePath: instancePath + "/alive",
						schemaPath: "#/properties/alive/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
		} else {
			const err11 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err11];
			else vErrors.push(err11);
			errors++;
		}
		validate82.errors = vErrors;
		return errors === 0;
	}
	function validate75(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate76(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate76.errors : vErrors.concat(validate76.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate82(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate82.errors : vErrors.concat(validate82.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs2 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else if (_valid0) {
			valid0 = true;
			passing0 = 1;
		}
		if (!valid0) {
			const err0 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate75.errors = vErrors;
		return errors === 0;
	}
	var schema129 = {
		"type": "object",
		"additionalProperties": false,
		"required": [
			"id",
			"cell",
			"owner",
			"range",
			"placedAt",
			"trigger",
			"blast"
		],
		"properties": {
			"id": { "$ref": "#/$defs/EntityId" },
			"cell": { "$ref": "#/$defs/Cell" },
			"owner": { "$ref": "#/$defs/EntityId" },
			"range": {
				"type": "integer",
				"minimum": 1,
				"maximum": 8
			},
			"placedAt": { "$ref": "#/$defs/Tick" },
			"trigger": { "$ref": "#/$defs/BombTrigger" },
			"blast": { "enum": [
				"standard",
				"piercing",
				"power"
			] }
		}
	};
	function validate86(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/$defs/TimedBombTrigger/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.remainingTicks === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/$defs/TimedBombTrigger/required",
					keyword: "required",
					params: { missingProperty: "remainingTicks" },
					message: "must have required property 'remainingTicks'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.paused === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/$defs/TimedBombTrigger/required",
					keyword: "required",
					params: { missingProperty: "paused" },
					message: "must have required property 'paused'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "remainingTicks" || key0 === "paused")) {
				const err3 = {
					instancePath,
					schemaPath: "#/$defs/TimedBombTrigger/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.type !== void 0) {
				if ("timed" !== data.type) {
					const err4 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/$defs/TimedBombTrigger/properties/type/const",
						keyword: "const",
						params: { allowedValue: "timed" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.remainingTicks !== void 0) {
				let data1 = data.remainingTicks;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err5 = {
						instancePath: instancePath + "/remainingTicks",
						schemaPath: "#/$defs/TimedBombTrigger/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 600 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/$defs/TimedBombTrigger/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 600
							},
							message: "must be <= 600"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/$defs/TimedBombTrigger/properties/remainingTicks/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
				}
			}
			if (data.paused !== void 0) {
				if (typeof data.paused !== "boolean") {
					const err8 = {
						instancePath: instancePath + "/paused",
						schemaPath: "#/$defs/TimedBombTrigger/properties/paused/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
			}
		} else {
			const err9 = {
				instancePath,
				schemaPath: "#/$defs/TimedBombTrigger/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err9];
			else vErrors.push(err9);
			errors++;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs10 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err10 = {
					instancePath,
					schemaPath: "#/$defs/RemoteBombTrigger/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err10];
				else vErrors.push(err10);
				errors++;
			}
			for (const key1 in data) if (!(key1 === "type")) {
				const err11 = {
					instancePath,
					schemaPath: "#/$defs/RemoteBombTrigger/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key1 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err11];
				else vErrors.push(err11);
				errors++;
			}
			if (data.type !== void 0) {
				if ("remote" !== data.type) {
					const err12 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/$defs/RemoteBombTrigger/properties/type/const",
						keyword: "const",
						params: { allowedValue: "remote" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				}
			}
		} else {
			const err13 = {
				instancePath,
				schemaPath: "#/$defs/RemoteBombTrigger/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err13];
			else vErrors.push(err13);
			errors++;
		}
		var _valid0 = _errs10 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else {
			if (_valid0) {
				valid0 = true;
				passing0 = 1;
			}
			const _errs15 = errors;
			if (data && typeof data == "object" && !Array.isArray(data)) {
				if (data.type === void 0) {
					const err14 = {
						instancePath,
						schemaPath: "#/$defs/ContactBombTrigger/required",
						keyword: "required",
						params: { missingProperty: "type" },
						message: "must have required property 'type'"
					};
					if (vErrors === null) vErrors = [err14];
					else vErrors.push(err14);
					errors++;
				}
				for (const key2 in data) if (!(key2 === "type")) {
					const err15 = {
						instancePath,
						schemaPath: "#/$defs/ContactBombTrigger/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key2 },
						message: "must NOT have additional properties"
					};
					if (vErrors === null) vErrors = [err15];
					else vErrors.push(err15);
					errors++;
				}
				if (data.type !== void 0) {
					if ("contact" !== data.type) {
						const err16 = {
							instancePath: instancePath + "/type",
							schemaPath: "#/$defs/ContactBombTrigger/properties/type/const",
							keyword: "const",
							params: { allowedValue: "contact" },
							message: "must be equal to constant"
						};
						if (vErrors === null) vErrors = [err16];
						else vErrors.push(err16);
						errors++;
					}
				}
			} else {
				const err17 = {
					instancePath,
					schemaPath: "#/$defs/ContactBombTrigger/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err17];
				else vErrors.push(err17);
				errors++;
			}
			var _valid0 = _errs15 === errors;
			if (_valid0 && valid0) {
				valid0 = false;
				passing0 = [passing0, 2];
			} else if (_valid0) {
				valid0 = true;
				passing0 = 2;
			}
		}
		if (!valid0) {
			const err18 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err18];
			else vErrors.push(err18);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate86.errors = vErrors;
		return errors === 0;
	}
	function validate85(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.id === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "id" },
					message: "must have required property 'id'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.cell === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "cell" },
					message: "must have required property 'cell'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.owner === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "owner" },
					message: "must have required property 'owner'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.range === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "range" },
					message: "must have required property 'range'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.placedAt === void 0) {
				const err4 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "placedAt" },
					message: "must have required property 'placedAt'"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.trigger === void 0) {
				const err5 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "trigger" },
					message: "must have required property 'trigger'"
				};
				if (vErrors === null) vErrors = [err5];
				else vErrors.push(err5);
				errors++;
			}
			if (data.blast === void 0) {
				const err6 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "blast" },
					message: "must have required property 'blast'"
				};
				if (vErrors === null) vErrors = [err6];
				else vErrors.push(err6);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "id" || key0 === "cell" || key0 === "owner" || key0 === "range" || key0 === "placedAt" || key0 === "trigger" || key0 === "blast")) {
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
			if (data.id !== void 0) {
				let data0 = data.id;
				if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0) && isFinite(data0))) {
					const err8 = {
						instancePath: instancePath + "/id",
						schemaPath: "#/$defs/EntityId/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
				if (typeof data0 == "number" && isFinite(data0)) {
					if (data0 > 4294967295 || isNaN(data0)) {
						const err9 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 4294967295
							},
							message: "must be <= 4294967295"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
					if (data0 < 0 || isNaN(data0)) {
						const err10 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/minimum",
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
			if (data.cell !== void 0) {
				let data1 = data.cell;
				if (data1 && typeof data1 == "object" && !Array.isArray(data1)) {
					if (data1.column === void 0) {
						const err11 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/required",
							keyword: "required",
							params: { missingProperty: "column" },
							message: "must have required property 'column'"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
					if (data1.row === void 0) {
						const err12 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/required",
							keyword: "required",
							params: { missingProperty: "row" },
							message: "must have required property 'row'"
						};
						if (vErrors === null) vErrors = [err12];
						else vErrors.push(err12);
						errors++;
					}
					for (const key1 in data1) if (!(key1 === "column" || key1 === "row")) {
						const err13 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err13];
						else vErrors.push(err13);
						errors++;
					}
					if (data1.column !== void 0) {
						let data2 = data1.column;
						if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
							const err14 = {
								instancePath: instancePath + "/cell/column",
								schemaPath: "#/$defs/Cell/properties/column/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err14];
							else vErrors.push(err14);
							errors++;
						}
						if (typeof data2 == "number" && isFinite(data2)) {
							if (data2 > 14 || isNaN(data2)) {
								const err15 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/Cell/properties/column/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 14
									},
									message: "must be <= 14"
								};
								if (vErrors === null) vErrors = [err15];
								else vErrors.push(err15);
								errors++;
							}
							if (data2 < 0 || isNaN(data2)) {
								const err16 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/Cell/properties/column/minimum",
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
					if (data1.row !== void 0) {
						let data3 = data1.row;
						if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3) && isFinite(data3))) {
							const err17 = {
								instancePath: instancePath + "/cell/row",
								schemaPath: "#/$defs/Cell/properties/row/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err17];
							else vErrors.push(err17);
							errors++;
						}
						if (typeof data3 == "number" && isFinite(data3)) {
							if (data3 > 12 || isNaN(data3)) {
								const err18 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/Cell/properties/row/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 12
									},
									message: "must be <= 12"
								};
								if (vErrors === null) vErrors = [err18];
								else vErrors.push(err18);
								errors++;
							}
							if (data3 < 0 || isNaN(data3)) {
								const err19 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/Cell/properties/row/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err19];
								else vErrors.push(err19);
								errors++;
							}
						}
					}
				} else {
					const err20 = {
						instancePath: instancePath + "/cell",
						schemaPath: "#/$defs/Cell/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err20];
					else vErrors.push(err20);
					errors++;
				}
			}
			if (data.owner !== void 0) {
				let data4 = data.owner;
				if (!(typeof data4 == "number" && !(data4 % 1) && !isNaN(data4) && isFinite(data4))) {
					const err21 = {
						instancePath: instancePath + "/owner",
						schemaPath: "#/$defs/EntityId/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err21];
					else vErrors.push(err21);
					errors++;
				}
				if (typeof data4 == "number" && isFinite(data4)) {
					if (data4 > 4294967295 || isNaN(data4)) {
						const err22 = {
							instancePath: instancePath + "/owner",
							schemaPath: "#/$defs/EntityId/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 4294967295
							},
							message: "must be <= 4294967295"
						};
						if (vErrors === null) vErrors = [err22];
						else vErrors.push(err22);
						errors++;
					}
					if (data4 < 0 || isNaN(data4)) {
						const err23 = {
							instancePath: instancePath + "/owner",
							schemaPath: "#/$defs/EntityId/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err23];
						else vErrors.push(err23);
						errors++;
					}
				}
			}
			if (data.range !== void 0) {
				let data5 = data.range;
				if (!(typeof data5 == "number" && !(data5 % 1) && !isNaN(data5) && isFinite(data5))) {
					const err24 = {
						instancePath: instancePath + "/range",
						schemaPath: "#/properties/range/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err24];
					else vErrors.push(err24);
					errors++;
				}
				if (typeof data5 == "number" && isFinite(data5)) {
					if (data5 > 8 || isNaN(data5)) {
						const err25 = {
							instancePath: instancePath + "/range",
							schemaPath: "#/properties/range/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 8
							},
							message: "must be <= 8"
						};
						if (vErrors === null) vErrors = [err25];
						else vErrors.push(err25);
						errors++;
					}
					if (data5 < 1 || isNaN(data5)) {
						const err26 = {
							instancePath: instancePath + "/range",
							schemaPath: "#/properties/range/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err26];
						else vErrors.push(err26);
						errors++;
					}
				}
			}
			if (data.placedAt !== void 0) {
				let data6 = data.placedAt;
				if (!(typeof data6 == "number" && !(data6 % 1) && !isNaN(data6) && isFinite(data6))) {
					const err27 = {
						instancePath: instancePath + "/placedAt",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err27];
					else vErrors.push(err27);
					errors++;
				}
				if (typeof data6 == "number" && isFinite(data6)) {
					if (data6 > 9007199254740991 || isNaN(data6)) {
						const err28 = {
							instancePath: instancePath + "/placedAt",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err28];
						else vErrors.push(err28);
						errors++;
					}
					if (data6 < 0 || isNaN(data6)) {
						const err29 = {
							instancePath: instancePath + "/placedAt",
							schemaPath: "#/$defs/Tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err29];
						else vErrors.push(err29);
						errors++;
					}
				}
			}
			if (data.trigger !== void 0) {
				if (!validate86(data.trigger, {
					instancePath: instancePath + "/trigger",
					parentData: data,
					parentDataProperty: "trigger",
					rootData
				})) {
					vErrors = vErrors === null ? validate86.errors : vErrors.concat(validate86.errors);
					errors = vErrors.length;
				}
			}
			if (data.blast !== void 0) {
				let data8 = data.blast;
				if (!(data8 === "standard" || data8 === "piercing" || data8 === "power")) {
					const err30 = {
						instancePath: instancePath + "/blast",
						schemaPath: "#/properties/blast/enum",
						keyword: "enum",
						params: { allowedValues: schema129.properties.blast.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err30];
					else vErrors.push(err30);
					errors++;
				}
			}
		} else {
			const err31 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err31];
			else vErrors.push(err31);
			errors++;
		}
		validate85.errors = vErrors;
		return errors === 0;
	}
	function validate89(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.cell === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "cell" },
					message: "must have required property 'cell'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.expiresAt === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "expiresAt" },
					message: "must have required property 'expiresAt'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "cell" || key0 === "expiresAt")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.cell !== void 0) {
				let data0 = data.cell;
				if (data0 && typeof data0 == "object" && !Array.isArray(data0)) {
					if (data0.column === void 0) {
						const err3 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/required",
							keyword: "required",
							params: { missingProperty: "column" },
							message: "must have required property 'column'"
						};
						if (vErrors === null) vErrors = [err3];
						else vErrors.push(err3);
						errors++;
					}
					if (data0.row === void 0) {
						const err4 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/required",
							keyword: "required",
							params: { missingProperty: "row" },
							message: "must have required property 'row'"
						};
						if (vErrors === null) vErrors = [err4];
						else vErrors.push(err4);
						errors++;
					}
					for (const key1 in data0) if (!(key1 === "column" || key1 === "row")) {
						const err5 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data0.column !== void 0) {
						let data1 = data0.column;
						if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
							const err6 = {
								instancePath: instancePath + "/cell/column",
								schemaPath: "#/$defs/Cell/properties/column/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err6];
							else vErrors.push(err6);
							errors++;
						}
						if (typeof data1 == "number" && isFinite(data1)) {
							if (data1 > 14 || isNaN(data1)) {
								const err7 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/Cell/properties/column/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 14
									},
									message: "must be <= 14"
								};
								if (vErrors === null) vErrors = [err7];
								else vErrors.push(err7);
								errors++;
							}
							if (data1 < 0 || isNaN(data1)) {
								const err8 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/Cell/properties/column/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err8];
								else vErrors.push(err8);
								errors++;
							}
						}
					}
					if (data0.row !== void 0) {
						let data2 = data0.row;
						if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
							const err9 = {
								instancePath: instancePath + "/cell/row",
								schemaPath: "#/$defs/Cell/properties/row/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err9];
							else vErrors.push(err9);
							errors++;
						}
						if (typeof data2 == "number" && isFinite(data2)) {
							if (data2 > 12 || isNaN(data2)) {
								const err10 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/Cell/properties/row/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 12
									},
									message: "must be <= 12"
								};
								if (vErrors === null) vErrors = [err10];
								else vErrors.push(err10);
								errors++;
							}
							if (data2 < 0 || isNaN(data2)) {
								const err11 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/Cell/properties/row/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err11];
								else vErrors.push(err11);
								errors++;
							}
						}
					}
				} else {
					const err12 = {
						instancePath: instancePath + "/cell",
						schemaPath: "#/$defs/Cell/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				}
			}
			if (data.expiresAt !== void 0) {
				let data3 = data.expiresAt;
				if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3) && isFinite(data3))) {
					const err13 = {
						instancePath: instancePath + "/expiresAt",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err13];
					else vErrors.push(err13);
					errors++;
				}
				if (typeof data3 == "number" && isFinite(data3)) {
					if (data3 > 9007199254740991 || isNaN(data3)) {
						const err14 = {
							instancePath: instancePath + "/expiresAt",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err14];
						else vErrors.push(err14);
						errors++;
					}
					if (data3 < 0 || isNaN(data3)) {
						const err15 = {
							instancePath: instancePath + "/expiresAt",
							schemaPath: "#/$defs/Tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err15];
						else vErrors.push(err15);
						errors++;
					}
				}
			}
		} else {
			const err16 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err16];
			else vErrors.push(err16);
			errors++;
		}
		validate89.errors = vErrors;
		return errors === 0;
	}
	var schema143 = { "enum": [
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
	] };
	function validate91(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.cell === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "cell" },
					message: "must have required property 'cell'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.kind === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "kind" },
					message: "must have required property 'kind'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "cell" || key0 === "kind")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.cell !== void 0) {
				let data0 = data.cell;
				if (data0 && typeof data0 == "object" && !Array.isArray(data0)) {
					if (data0.column === void 0) {
						const err3 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/required",
							keyword: "required",
							params: { missingProperty: "column" },
							message: "must have required property 'column'"
						};
						if (vErrors === null) vErrors = [err3];
						else vErrors.push(err3);
						errors++;
					}
					if (data0.row === void 0) {
						const err4 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/required",
							keyword: "required",
							params: { missingProperty: "row" },
							message: "must have required property 'row'"
						};
						if (vErrors === null) vErrors = [err4];
						else vErrors.push(err4);
						errors++;
					}
					for (const key1 in data0) if (!(key1 === "column" || key1 === "row")) {
						const err5 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data0.column !== void 0) {
						let data1 = data0.column;
						if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
							const err6 = {
								instancePath: instancePath + "/cell/column",
								schemaPath: "#/$defs/Cell/properties/column/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err6];
							else vErrors.push(err6);
							errors++;
						}
						if (typeof data1 == "number" && isFinite(data1)) {
							if (data1 > 14 || isNaN(data1)) {
								const err7 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/Cell/properties/column/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 14
									},
									message: "must be <= 14"
								};
								if (vErrors === null) vErrors = [err7];
								else vErrors.push(err7);
								errors++;
							}
							if (data1 < 0 || isNaN(data1)) {
								const err8 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/Cell/properties/column/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err8];
								else vErrors.push(err8);
								errors++;
							}
						}
					}
					if (data0.row !== void 0) {
						let data2 = data0.row;
						if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
							const err9 = {
								instancePath: instancePath + "/cell/row",
								schemaPath: "#/$defs/Cell/properties/row/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err9];
							else vErrors.push(err9);
							errors++;
						}
						if (typeof data2 == "number" && isFinite(data2)) {
							if (data2 > 12 || isNaN(data2)) {
								const err10 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/Cell/properties/row/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 12
									},
									message: "must be <= 12"
								};
								if (vErrors === null) vErrors = [err10];
								else vErrors.push(err10);
								errors++;
							}
							if (data2 < 0 || isNaN(data2)) {
								const err11 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/Cell/properties/row/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err11];
								else vErrors.push(err11);
								errors++;
							}
						}
					}
				} else {
					const err12 = {
						instancePath: instancePath + "/cell",
						schemaPath: "#/$defs/Cell/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				}
			}
			if (data.kind !== void 0) {
				let data3 = data.kind;
				if (!(data3 === "bombCapacity" || data3 === "blastRange" || data3 === "movementSpeed" || data3 === "kick" || data3 === "punch" || data3 === "bracelet" || data3 === "wallPass" || data3 === "heart" || data3 === "remoteBomb" || data3 === "piercingBomb" || data3 === "powerBomb" || data3 === "landmine" || data3 === "shield" || data3 === "stopwatch" || data3 === "invisibilityCloak" || data3 === "goldenFlame" || data3 === "shortFuse" || data3 === "diarrhea" || data3 === "slowFoot" || data3 === "shortFlame" || data3 === "longFuse" || data3 === "reverseControls" || data3 === "invisibleBombs" || data3 === "oilBarrel" || data3 === "teleporter")) {
					const err13 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/$defs/PickupKind/enum",
						keyword: "enum",
						params: { allowedValues: schema143.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err13];
					else vErrors.push(err13);
					errors++;
				}
			}
		} else {
			const err14 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err14];
			else vErrors.push(err14);
			errors++;
		}
		validate91.errors = vErrors;
		return errors === 0;
	}
	var schema146 = { "enum": ["unlit", "burning"] };
	function validate93(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.cell === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "cell" },
					message: "must have required property 'cell'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.state === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "state" },
					message: "must have required property 'state'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "cell" || key0 === "state")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.cell !== void 0) {
				let data0 = data.cell;
				if (data0 && typeof data0 == "object" && !Array.isArray(data0)) {
					if (data0.column === void 0) {
						const err3 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/required",
							keyword: "required",
							params: { missingProperty: "column" },
							message: "must have required property 'column'"
						};
						if (vErrors === null) vErrors = [err3];
						else vErrors.push(err3);
						errors++;
					}
					if (data0.row === void 0) {
						const err4 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/required",
							keyword: "required",
							params: { missingProperty: "row" },
							message: "must have required property 'row'"
						};
						if (vErrors === null) vErrors = [err4];
						else vErrors.push(err4);
						errors++;
					}
					for (const key1 in data0) if (!(key1 === "column" || key1 === "row")) {
						const err5 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data0.column !== void 0) {
						let data1 = data0.column;
						if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
							const err6 = {
								instancePath: instancePath + "/cell/column",
								schemaPath: "#/$defs/Cell/properties/column/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err6];
							else vErrors.push(err6);
							errors++;
						}
						if (typeof data1 == "number" && isFinite(data1)) {
							if (data1 > 14 || isNaN(data1)) {
								const err7 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/Cell/properties/column/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 14
									},
									message: "must be <= 14"
								};
								if (vErrors === null) vErrors = [err7];
								else vErrors.push(err7);
								errors++;
							}
							if (data1 < 0 || isNaN(data1)) {
								const err8 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/Cell/properties/column/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err8];
								else vErrors.push(err8);
								errors++;
							}
						}
					}
					if (data0.row !== void 0) {
						let data2 = data0.row;
						if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
							const err9 = {
								instancePath: instancePath + "/cell/row",
								schemaPath: "#/$defs/Cell/properties/row/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err9];
							else vErrors.push(err9);
							errors++;
						}
						if (typeof data2 == "number" && isFinite(data2)) {
							if (data2 > 12 || isNaN(data2)) {
								const err10 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/Cell/properties/row/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 12
									},
									message: "must be <= 12"
								};
								if (vErrors === null) vErrors = [err10];
								else vErrors.push(err10);
								errors++;
							}
							if (data2 < 0 || isNaN(data2)) {
								const err11 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/Cell/properties/row/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err11];
								else vErrors.push(err11);
								errors++;
							}
						}
					}
				} else {
					const err12 = {
						instancePath: instancePath + "/cell",
						schemaPath: "#/$defs/Cell/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				}
			}
			if (data.state !== void 0) {
				let data3 = data.state;
				if (!(data3 === "unlit" || data3 === "burning")) {
					const err13 = {
						instancePath: instancePath + "/state",
						schemaPath: "#/$defs/OilBarrelState/enum",
						keyword: "enum",
						params: { allowedValues: schema146.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err13];
					else vErrors.push(err13);
					errors++;
				}
			}
		} else {
			const err14 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err14];
			else vErrors.push(err14);
			errors++;
		}
		validate93.errors = vErrors;
		return errors === 0;
	}
	function validate95(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.position === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "position" },
					message: "must have required property 'position'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.direction === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "direction" },
					message: "must have required property 'direction'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.legRemainingTicks === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "legRemainingTicks" },
					message: "must have required property 'legRemainingTicks'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "position" || key0 === "direction" || key0 === "legRemainingTicks")) {
				const err3 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.position !== void 0) {
				let data0 = data.position;
				if (data0 && typeof data0 == "object" && !Array.isArray(data0)) {
					if (data0.x === void 0) {
						const err4 = {
							instancePath: instancePath + "/position",
							schemaPath: "#/$defs/Position/required",
							keyword: "required",
							params: { missingProperty: "x" },
							message: "must have required property 'x'"
						};
						if (vErrors === null) vErrors = [err4];
						else vErrors.push(err4);
						errors++;
					}
					if (data0.y === void 0) {
						const err5 = {
							instancePath: instancePath + "/position",
							schemaPath: "#/$defs/Position/required",
							keyword: "required",
							params: { missingProperty: "y" },
							message: "must have required property 'y'"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					for (const key1 in data0) if (!(key1 === "x" || key1 === "y")) {
						const err6 = {
							instancePath: instancePath + "/position",
							schemaPath: "#/$defs/Position/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					if (data0.x !== void 0) {
						let data1 = data0.x;
						if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
							const err7 = {
								instancePath: instancePath + "/position/x",
								schemaPath: "#/$defs/Position/properties/x/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err7];
							else vErrors.push(err7);
							errors++;
						}
						if (typeof data1 == "number" && isFinite(data1)) {
							if (data1 > 15359 || isNaN(data1)) {
								const err8 = {
									instancePath: instancePath + "/position/x",
									schemaPath: "#/$defs/Position/properties/x/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 15359
									},
									message: "must be <= 15359"
								};
								if (vErrors === null) vErrors = [err8];
								else vErrors.push(err8);
								errors++;
							}
							if (data1 < 0 || isNaN(data1)) {
								const err9 = {
									instancePath: instancePath + "/position/x",
									schemaPath: "#/$defs/Position/properties/x/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err9];
								else vErrors.push(err9);
								errors++;
							}
						}
					}
					if (data0.y !== void 0) {
						let data2 = data0.y;
						if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
							const err10 = {
								instancePath: instancePath + "/position/y",
								schemaPath: "#/$defs/Position/properties/y/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err10];
							else vErrors.push(err10);
							errors++;
						}
						if (typeof data2 == "number" && isFinite(data2)) {
							if (data2 > 13311 || isNaN(data2)) {
								const err11 = {
									instancePath: instancePath + "/position/y",
									schemaPath: "#/$defs/Position/properties/y/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 13311
									},
									message: "must be <= 13311"
								};
								if (vErrors === null) vErrors = [err11];
								else vErrors.push(err11);
								errors++;
							}
							if (data2 < 0 || isNaN(data2)) {
								const err12 = {
									instancePath: instancePath + "/position/y",
									schemaPath: "#/$defs/Position/properties/y/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err12];
								else vErrors.push(err12);
								errors++;
							}
						}
					}
				} else {
					const err13 = {
						instancePath: instancePath + "/position",
						schemaPath: "#/$defs/Position/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err13];
					else vErrors.push(err13);
					errors++;
				}
			}
			if (data.direction !== void 0) {
				let data3 = data.direction;
				if (!(data3 === "north" || data3 === "east" || data3 === "south" || data3 === "west")) {
					const err14 = {
						instancePath: instancePath + "/direction",
						schemaPath: "#/$defs/Direction/enum",
						keyword: "enum",
						params: { allowedValues: schema109.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err14];
					else vErrors.push(err14);
					errors++;
				}
			}
			if (data.legRemainingTicks !== void 0) {
				let data4 = data.legRemainingTicks;
				if (!(typeof data4 == "number" && !(data4 % 1) && !isNaN(data4) && isFinite(data4))) {
					const err15 = {
						instancePath: instancePath + "/legRemainingTicks",
						schemaPath: "#/properties/legRemainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err15];
					else vErrors.push(err15);
					errors++;
				}
				if (typeof data4 == "number" && isFinite(data4)) {
					if (data4 > 128 || isNaN(data4)) {
						const err16 = {
							instancePath: instancePath + "/legRemainingTicks",
							schemaPath: "#/properties/legRemainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 128
							},
							message: "must be <= 128"
						};
						if (vErrors === null) vErrors = [err16];
						else vErrors.push(err16);
						errors++;
					}
					if (data4 < 0 || isNaN(data4)) {
						const err17 = {
							instancePath: instancePath + "/legRemainingTicks",
							schemaPath: "#/properties/legRemainingTicks/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err17];
						else vErrors.push(err17);
						errors++;
					}
				}
			}
		} else {
			const err18 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err18];
			else vErrors.push(err18);
			errors++;
		}
		validate95.errors = vErrors;
		return errors === 0;
	}
	function validate66(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.runningUpdates === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "runningUpdates" },
					message: "must have required property 'runningUpdates'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.remainingTicks === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "remainingTicks" },
					message: "must have required property 'remainingTicks'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.physicalTick === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "physicalTick" },
					message: "must have required property 'physicalTick'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.arena === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "arena" },
					message: "must have required property 'arena'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.actors === void 0) {
				const err4 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "actors" },
					message: "must have required property 'actors'"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.bombs === void 0) {
				const err5 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "bombs" },
					message: "must have required property 'bombs'"
				};
				if (vErrors === null) vErrors = [err5];
				else vErrors.push(err5);
				errors++;
			}
			if (data.flames === void 0) {
				const err6 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "flames" },
					message: "must have required property 'flames'"
				};
				if (vErrors === null) vErrors = [err6];
				else vErrors.push(err6);
				errors++;
			}
			if (data.oilBarrels === void 0) {
				const err7 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "oilBarrels" },
					message: "must have required property 'oilBarrels'"
				};
				if (vErrors === null) vErrors = [err7];
				else vErrors.push(err7);
				errors++;
			}
			if (data.pickups === void 0) {
				const err8 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "pickups" },
					message: "must have required property 'pickups'"
				};
				if (vErrors === null) vErrors = [err8];
				else vErrors.push(err8);
				errors++;
			}
			if (data.stormCloud === void 0) {
				const err9 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "stormCloud" },
					message: "must have required property 'stormCloud'"
				};
				if (vErrors === null) vErrors = [err9];
				else vErrors.push(err9);
				errors++;
			}
			for (const key0 in data) if (!func2.call(schema92.properties, key0)) {
				const err10 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err10];
				else vErrors.push(err10);
				errors++;
			}
			if (data.runningUpdates !== void 0) {
				let data0 = data.runningUpdates;
				if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0) && isFinite(data0))) {
					const err11 = {
						instancePath: instancePath + "/runningUpdates",
						schemaPath: "#/properties/runningUpdates/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err11];
					else vErrors.push(err11);
					errors++;
				}
				if (typeof data0 == "number" && isFinite(data0)) {
					if (data0 > 10800 || isNaN(data0)) {
						const err12 = {
							instancePath: instancePath + "/runningUpdates",
							schemaPath: "#/properties/runningUpdates/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 10800
							},
							message: "must be <= 10800"
						};
						if (vErrors === null) vErrors = [err12];
						else vErrors.push(err12);
						errors++;
					}
					if (data0 < 0 || isNaN(data0)) {
						const err13 = {
							instancePath: instancePath + "/runningUpdates",
							schemaPath: "#/properties/runningUpdates/minimum",
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
			if (data.remainingTicks !== void 0) {
				let data1 = data.remainingTicks;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err14 = {
						instancePath: instancePath + "/remainingTicks",
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err14];
					else vErrors.push(err14);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 10800 || isNaN(data1)) {
						const err15 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 10800
							},
							message: "must be <= 10800"
						};
						if (vErrors === null) vErrors = [err15];
						else vErrors.push(err15);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err16 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
			if (data.physicalTick !== void 0) {
				let data2 = data.physicalTick;
				if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
					const err17 = {
						instancePath: instancePath + "/physicalTick",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err17];
					else vErrors.push(err17);
					errors++;
				}
				if (typeof data2 == "number" && isFinite(data2)) {
					if (data2 > 9007199254740991 || isNaN(data2)) {
						const err18 = {
							instancePath: instancePath + "/physicalTick",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err18];
						else vErrors.push(err18);
						errors++;
					}
					if (data2 < 0 || isNaN(data2)) {
						const err19 = {
							instancePath: instancePath + "/physicalTick",
							schemaPath: "#/$defs/Tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err19];
						else vErrors.push(err19);
						errors++;
					}
				}
			}
			if (data.arena !== void 0) {
				if (!validate67(data.arena, {
					instancePath: instancePath + "/arena",
					parentData: data,
					parentDataProperty: "arena",
					rootData
				})) {
					vErrors = vErrors === null ? validate67.errors : vErrors.concat(validate67.errors);
					errors = vErrors.length;
				}
			}
			if (data.actors !== void 0) {
				let data4 = data.actors;
				if (Array.isArray(data4)) {
					if (data4.length > 4) {
						const err20 = {
							instancePath: instancePath + "/actors",
							schemaPath: "#/properties/actors/maxItems",
							keyword: "maxItems",
							params: { limit: 4 },
							message: "must NOT have more than 4 items"
						};
						if (vErrors === null) vErrors = [err20];
						else vErrors.push(err20);
						errors++;
					}
					if (data4.length < 2) {
						const err21 = {
							instancePath: instancePath + "/actors",
							schemaPath: "#/properties/actors/minItems",
							keyword: "minItems",
							params: { limit: 2 },
							message: "must NOT have fewer than 2 items"
						};
						if (vErrors === null) vErrors = [err21];
						else vErrors.push(err21);
						errors++;
					}
					const len0 = data4.length;
					for (let i0 = 0; i0 < len0; i0++) if (!validate75(data4[i0], {
						instancePath: instancePath + "/actors/" + i0,
						parentData: data4,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate75.errors : vErrors.concat(validate75.errors);
						errors = vErrors.length;
					}
				} else {
					const err22 = {
						instancePath: instancePath + "/actors",
						schemaPath: "#/properties/actors/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err22];
					else vErrors.push(err22);
					errors++;
				}
			}
			if (data.bombs !== void 0) {
				let data6 = data.bombs;
				if (Array.isArray(data6)) {
					if (data6.length > 32) {
						const err23 = {
							instancePath: instancePath + "/bombs",
							schemaPath: "#/properties/bombs/maxItems",
							keyword: "maxItems",
							params: { limit: 32 },
							message: "must NOT have more than 32 items"
						};
						if (vErrors === null) vErrors = [err23];
						else vErrors.push(err23);
						errors++;
					}
					if (data6.length < 0) {
						const err24 = {
							instancePath: instancePath + "/bombs",
							schemaPath: "#/properties/bombs/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err24];
						else vErrors.push(err24);
						errors++;
					}
					const len1 = data6.length;
					for (let i1 = 0; i1 < len1; i1++) if (!validate85(data6[i1], {
						instancePath: instancePath + "/bombs/" + i1,
						parentData: data6,
						parentDataProperty: i1,
						rootData
					})) {
						vErrors = vErrors === null ? validate85.errors : vErrors.concat(validate85.errors);
						errors = vErrors.length;
					}
				} else {
					const err25 = {
						instancePath: instancePath + "/bombs",
						schemaPath: "#/properties/bombs/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err25];
					else vErrors.push(err25);
					errors++;
				}
			}
			if (data.flames !== void 0) {
				let data8 = data.flames;
				if (Array.isArray(data8)) {
					if (data8.length > 195) {
						const err26 = {
							instancePath: instancePath + "/flames",
							schemaPath: "#/properties/flames/maxItems",
							keyword: "maxItems",
							params: { limit: 195 },
							message: "must NOT have more than 195 items"
						};
						if (vErrors === null) vErrors = [err26];
						else vErrors.push(err26);
						errors++;
					}
					if (data8.length < 0) {
						const err27 = {
							instancePath: instancePath + "/flames",
							schemaPath: "#/properties/flames/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err27];
						else vErrors.push(err27);
						errors++;
					}
					const len2 = data8.length;
					for (let i2 = 0; i2 < len2; i2++) if (!validate89(data8[i2], {
						instancePath: instancePath + "/flames/" + i2,
						parentData: data8,
						parentDataProperty: i2,
						rootData
					})) {
						vErrors = vErrors === null ? validate89.errors : vErrors.concat(validate89.errors);
						errors = vErrors.length;
					}
				} else {
					const err28 = {
						instancePath: instancePath + "/flames",
						schemaPath: "#/properties/flames/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err28];
					else vErrors.push(err28);
					errors++;
				}
			}
			if (data.pickups !== void 0) {
				let data10 = data.pickups;
				if (Array.isArray(data10)) {
					if (data10.length > 195) {
						const err29 = {
							instancePath: instancePath + "/pickups",
							schemaPath: "#/properties/pickups/maxItems",
							keyword: "maxItems",
							params: { limit: 195 },
							message: "must NOT have more than 195 items"
						};
						if (vErrors === null) vErrors = [err29];
						else vErrors.push(err29);
						errors++;
					}
					if (data10.length < 0) {
						const err30 = {
							instancePath: instancePath + "/pickups",
							schemaPath: "#/properties/pickups/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err30];
						else vErrors.push(err30);
						errors++;
					}
					const len3 = data10.length;
					for (let i3 = 0; i3 < len3; i3++) if (!validate91(data10[i3], {
						instancePath: instancePath + "/pickups/" + i3,
						parentData: data10,
						parentDataProperty: i3,
						rootData
					})) {
						vErrors = vErrors === null ? validate91.errors : vErrors.concat(validate91.errors);
						errors = vErrors.length;
					}
					let i4 = data10.length;
					let j0;
					if (i4 > 1) {
						outer0: for (; i4--;) for (j0 = i4; j0--;) if (func0(data10[i4], data10[j0])) {
							const err31 = {
								instancePath: instancePath + "/pickups",
								schemaPath: "#/properties/pickups/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i4,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i4 + " are identical)"
							};
							if (vErrors === null) vErrors = [err31];
							else vErrors.push(err31);
							errors++;
							break outer0;
						}
					}
				} else {
					const err32 = {
						instancePath: instancePath + "/pickups",
						schemaPath: "#/properties/pickups/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err32];
					else vErrors.push(err32);
					errors++;
				}
			}
			if (data.oilBarrels !== void 0) {
				let data12 = data.oilBarrels;
				if (Array.isArray(data12)) {
					if (data12.length > 8) {
						const err33 = {
							instancePath: instancePath + "/oilBarrels",
							schemaPath: "#/properties/oilBarrels/maxItems",
							keyword: "maxItems",
							params: { limit: 8 },
							message: "must NOT have more than 8 items"
						};
						if (vErrors === null) vErrors = [err33];
						else vErrors.push(err33);
						errors++;
					}
					if (data12.length < 0) {
						const err34 = {
							instancePath: instancePath + "/oilBarrels",
							schemaPath: "#/properties/oilBarrels/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err34];
						else vErrors.push(err34);
						errors++;
					}
					const len4 = data12.length;
					for (let i5 = 0; i5 < len4; i5++) if (!validate93(data12[i5], {
						instancePath: instancePath + "/oilBarrels/" + i5,
						parentData: data12,
						parentDataProperty: i5,
						rootData
					})) {
						vErrors = vErrors === null ? validate93.errors : vErrors.concat(validate93.errors);
						errors = vErrors.length;
					}
					let i6 = data12.length;
					let j1;
					if (i6 > 1) {
						outer1: for (; i6--;) for (j1 = i6; j1--;) if (func0(data12[i6], data12[j1])) {
							const err35 = {
								instancePath: instancePath + "/oilBarrels",
								schemaPath: "#/properties/oilBarrels/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i6,
									j: j1
								},
								message: "must NOT have duplicate items (items ## " + j1 + " and " + i6 + " are identical)"
							};
							if (vErrors === null) vErrors = [err35];
							else vErrors.push(err35);
							errors++;
							break outer1;
						}
					}
				} else {
					const err36 = {
						instancePath: instancePath + "/oilBarrels",
						schemaPath: "#/properties/oilBarrels/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err36];
					else vErrors.push(err36);
					errors++;
				}
			}
			if (data.stormCloud !== void 0) {
				let data14 = data.stormCloud;
				const _errs26 = errors;
				let valid14 = false;
				let passing0 = null;
				const _errs27 = errors;
				if (!validate95(data14, {
					instancePath: instancePath + "/stormCloud",
					parentData: data,
					parentDataProperty: "stormCloud",
					rootData
				})) {
					vErrors = vErrors === null ? validate95.errors : vErrors.concat(validate95.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs27 === errors;
				if (_valid0) {
					valid14 = true;
					passing0 = 0;
				}
				const _errs28 = errors;
				if (data14 !== null) {
					const err37 = {
						instancePath: instancePath + "/stormCloud",
						schemaPath: "#/properties/stormCloud/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err37];
					else vErrors.push(err37);
					errors++;
				}
				var _valid0 = _errs28 === errors;
				if (_valid0 && valid14) {
					valid14 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid14 = true;
					passing0 = 1;
				}
				if (!valid14) {
					const err38 = {
						instancePath: instancePath + "/stormCloud",
						schemaPath: "#/properties/stormCloud/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err38];
					else vErrors.push(err38);
					errors++;
				} else {
					errors = _errs26;
					if (vErrors !== null) {
						if (_errs26) vErrors.length = _errs26;
						else vErrors = null;
					}
				}
			}
		} else {
			const err39 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err39];
			else vErrors.push(err39);
			errors++;
		}
		validate66.errors = vErrors;
		return errors === 0;
	}
	var pattern2 = /* @__PURE__ */ new RegExp("^[0-9a-f]{64}$", "u");
	function validate16(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.sequence === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "sequence" },
					message: "must have required property 'sequence'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.room === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "room" },
					message: "must have required property 'room'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.youSeat === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "youSeat" },
					message: "must have required property 'youSeat'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.resumeToken === void 0) {
				const err4 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "resumeToken" },
					message: "must have required property 'resumeToken'"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.lastActionId === void 0) {
				const err5 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "lastActionId" },
					message: "must have required property 'lastActionId'"
				};
				if (vErrors === null) vErrors = [err5];
				else vErrors.push(err5);
				errors++;
			}
			if (data.state === void 0) {
				const err6 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "state" },
					message: "must have required property 'state'"
				};
				if (vErrors === null) vErrors = [err6];
				else vErrors.push(err6);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "sequence" || key0 === "room" || key0 === "youSeat" || key0 === "resumeToken" || key0 === "state" || key0 === "lastActionId")) {
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
			if (data.type !== void 0) {
				if ("admitted" !== data.type) {
					const err8 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "admitted" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
			}
			if (data.sequence !== void 0) {
				let data1 = data.sequence;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err9 = {
						instancePath: instancePath + "/sequence",
						schemaPath: "#/$defs/Counter/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 2147483647 || isNaN(data1)) {
						const err10 = {
							instancePath: instancePath + "/sequence",
							schemaPath: "#/$defs/Counter/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err10];
						else vErrors.push(err10);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err11 = {
							instancePath: instancePath + "/sequence",
							schemaPath: "#/$defs/Counter/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
				}
			}
			if (data.room !== void 0) {
				if (!validate17(data.room, {
					instancePath: instancePath + "/room",
					parentData: data,
					parentDataProperty: "room",
					rootData
				})) {
					vErrors = vErrors === null ? validate17.errors : vErrors.concat(validate17.errors);
					errors = vErrors.length;
				}
			}
			if (data.youSeat !== void 0) {
				let data3 = data.youSeat;
				if (!(data3 === "one" || data3 === "two" || data3 === "three" || data3 === "four")) {
					const err12 = {
						instancePath: instancePath + "/youSeat",
						schemaPath: "#/$defs/Seat/enum",
						keyword: "enum",
						params: { allowedValues: schema30.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				}
			}
			if (data.resumeToken !== void 0) {
				let data4 = data.resumeToken;
				if (typeof data4 === "string") {
					if (func3(data4) > 64) {
						const err13 = {
							instancePath: instancePath + "/resumeToken",
							schemaPath: "#/$defs/ResumeToken/maxLength",
							keyword: "maxLength",
							params: { limit: 64 },
							message: "must NOT have more than 64 characters"
						};
						if (vErrors === null) vErrors = [err13];
						else vErrors.push(err13);
						errors++;
					}
					if (func3(data4) < 64) {
						const err14 = {
							instancePath: instancePath + "/resumeToken",
							schemaPath: "#/$defs/ResumeToken/minLength",
							keyword: "minLength",
							params: { limit: 64 },
							message: "must NOT have fewer than 64 characters"
						};
						if (vErrors === null) vErrors = [err14];
						else vErrors.push(err14);
						errors++;
					}
					if (!pattern2.test(data4)) {
						const err15 = {
							instancePath: instancePath + "/resumeToken",
							schemaPath: "#/$defs/ResumeToken/pattern",
							keyword: "pattern",
							params: { pattern: "^[0-9a-f]{64}$" },
							message: "must match pattern \"^[0-9a-f]{64}$\""
						};
						if (vErrors === null) vErrors = [err15];
						else vErrors.push(err15);
						errors++;
					}
				} else {
					const err16 = {
						instancePath: instancePath + "/resumeToken",
						schemaPath: "#/$defs/ResumeToken/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err16];
					else vErrors.push(err16);
					errors++;
				}
			}
			if (data.state !== void 0) {
				let data5 = data.state;
				const _errs13 = errors;
				let valid4 = false;
				let passing0 = null;
				const _errs14 = errors;
				if (!validate66(data5, {
					instancePath: instancePath + "/state",
					parentData: data,
					parentDataProperty: "state",
					rootData
				})) {
					vErrors = vErrors === null ? validate66.errors : vErrors.concat(validate66.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs14 === errors;
				if (_valid0) {
					valid4 = true;
					passing0 = 0;
				}
				const _errs15 = errors;
				if (data5 !== null) {
					const err17 = {
						instancePath: instancePath + "/state",
						schemaPath: "#/properties/state/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err17];
					else vErrors.push(err17);
					errors++;
				}
				var _valid0 = _errs15 === errors;
				if (_valid0 && valid4) {
					valid4 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid4 = true;
					passing0 = 1;
				}
				if (!valid4) {
					const err18 = {
						instancePath: instancePath + "/state",
						schemaPath: "#/properties/state/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err18];
					else vErrors.push(err18);
					errors++;
				} else {
					errors = _errs13;
					if (vErrors !== null) {
						if (_errs13) vErrors.length = _errs13;
						else vErrors = null;
					}
				}
			}
			if (data.lastActionId !== void 0) {
				let data6 = data.lastActionId;
				const _errs18 = errors;
				let valid5 = false;
				let passing1 = null;
				const _errs19 = errors;
				if (!(typeof data6 == "number" && !(data6 % 1) && !isNaN(data6) && isFinite(data6))) {
					const err19 = {
						instancePath: instancePath + "/lastActionId",
						schemaPath: "#/$defs/Counter/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err19];
					else vErrors.push(err19);
					errors++;
				}
				if (typeof data6 == "number" && isFinite(data6)) {
					if (data6 > 2147483647 || isNaN(data6)) {
						const err20 = {
							instancePath: instancePath + "/lastActionId",
							schemaPath: "#/$defs/Counter/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err20];
						else vErrors.push(err20);
						errors++;
					}
					if (data6 < 1 || isNaN(data6)) {
						const err21 = {
							instancePath: instancePath + "/lastActionId",
							schemaPath: "#/$defs/Counter/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err21];
						else vErrors.push(err21);
						errors++;
					}
				}
				var _valid1 = _errs19 === errors;
				if (_valid1) {
					valid5 = true;
					passing1 = 0;
				}
				const _errs22 = errors;
				if (data6 !== null) {
					const err22 = {
						instancePath: instancePath + "/lastActionId",
						schemaPath: "#/properties/lastActionId/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err22];
					else vErrors.push(err22);
					errors++;
				}
				var _valid1 = _errs22 === errors;
				if (_valid1 && valid5) {
					valid5 = false;
					passing1 = [passing1, 1];
				} else if (_valid1) {
					valid5 = true;
					passing1 = 1;
				}
				if (!valid5) {
					const err23 = {
						instancePath: instancePath + "/lastActionId",
						schemaPath: "#/properties/lastActionId/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing1 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err23];
					else vErrors.push(err23);
					errors++;
				} else {
					errors = _errs18;
					if (vErrors !== null) {
						if (_errs18) vErrors.length = _errs18;
						else vErrors = null;
					}
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
		validate16.errors = vErrors;
		return errors === 0;
	}
	function validate99(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.room === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "room" },
					message: "must have required property 'room'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "room")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("roomState" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "roomState" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.room !== void 0) {
				if (!validate17(data.room, {
					instancePath: instancePath + "/room",
					parentData: data,
					parentDataProperty: "room",
					rootData
				})) {
					vErrors = vErrors === null ? validate17.errors : vErrors.concat(validate17.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err4 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err4];
			else vErrors.push(err4);
			errors++;
		}
		validate99.errors = vErrors;
		return errors === 0;
	}
	function validate107(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.bomb === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "bomb" },
					message: "must have required property 'bomb'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "bomb")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("bombPlaced" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "bombPlaced" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.bomb !== void 0) {
				if (!validate85(data.bomb, {
					instancePath: instancePath + "/bomb",
					parentData: data,
					parentDataProperty: "bomb",
					rootData
				})) {
					vErrors = vErrors === null ? validate85.errors : vErrors.concat(validate85.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err4 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err4];
			else vErrors.push(err4);
			errors++;
		}
		validate107.errors = vErrors;
		return errors === 0;
	}
	function validate110(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.bomb === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "bomb" },
					message: "must have required property 'bomb'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "bomb")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("bombDetonated" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "bombDetonated" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.bomb !== void 0) {
				if (!validate85(data.bomb, {
					instancePath: instancePath + "/bomb",
					parentData: data,
					parentDataProperty: "bomb",
					rootData
				})) {
					vErrors = vErrors === null ? validate85.errors : vErrors.concat(validate85.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err4 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err4];
			else vErrors.push(err4);
			errors++;
		}
		validate110.errors = vErrors;
		return errors === 0;
	}
	function validate113(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.id === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "id" },
					message: "must have required property 'id'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.seat === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "seat" },
					message: "must have required property 'seat'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "id" || key0 === "seat")) {
				const err3 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.type !== void 0) {
				if ("actorEliminated" !== data.type) {
					const err4 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "actorEliminated" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.id !== void 0) {
				let data1 = data.id;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err5 = {
						instancePath: instancePath + "/id",
						schemaPath: "#/$defs/EntityId/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 4294967295 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 4294967295
							},
							message: "must be <= 4294967295"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
				}
			}
			if (data.seat !== void 0) {
				let data2 = data.seat;
				if (!(data2 === "one" || data2 === "two" || data2 === "three" || data2 === "four")) {
					const err8 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/Seat/enum",
						keyword: "enum",
						params: { allowedValues: schema30.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
			}
		} else {
			const err9 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err9];
			else vErrors.push(err9);
			errors++;
		}
		validate113.errors = vErrors;
		return errors === 0;
	}
	function validate115(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.result === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "result" },
					message: "must have required property 'result'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "result")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("roundEnded" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "roundEnded" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.result !== void 0) {
				if (!validate23(data.result, {
					instancePath: instancePath + "/result",
					parentData: data,
					parentDataProperty: "result",
					rootData
				})) {
					vErrors = vErrors === null ? validate23.errors : vErrors.concat(validate23.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err4 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err4];
			else vErrors.push(err4);
			errors++;
		}
		validate115.errors = vErrors;
		return errors === 0;
	}
	function validate118(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.id === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "id" },
					message: "must have required property 'id'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.seat === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "seat" },
					message: "must have required property 'seat'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.cell === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "cell" },
					message: "must have required property 'cell'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.kind === void 0) {
				const err4 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "kind" },
					message: "must have required property 'kind'"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "id" || key0 === "seat" || key0 === "cell" || key0 === "kind")) {
				const err5 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err5];
				else vErrors.push(err5);
				errors++;
			}
			if (data.type !== void 0) {
				if ("pickupCollected" !== data.type) {
					const err6 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "pickupCollected" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
			}
			if (data.id !== void 0) {
				let data1 = data.id;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err7 = {
						instancePath: instancePath + "/id",
						schemaPath: "#/$defs/EntityId/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err7];
					else vErrors.push(err7);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 4294967295 || isNaN(data1)) {
						const err8 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 4294967295
							},
							message: "must be <= 4294967295"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err9 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
				}
			}
			if (data.seat !== void 0) {
				let data2 = data.seat;
				if (!(data2 === "one" || data2 === "two" || data2 === "three" || data2 === "four")) {
					const err10 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/Seat/enum",
						keyword: "enum",
						params: { allowedValues: schema30.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
			if (data.cell !== void 0) {
				let data3 = data.cell;
				if (data3 && typeof data3 == "object" && !Array.isArray(data3)) {
					if (data3.column === void 0) {
						const err11 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/required",
							keyword: "required",
							params: { missingProperty: "column" },
							message: "must have required property 'column'"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
					if (data3.row === void 0) {
						const err12 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/required",
							keyword: "required",
							params: { missingProperty: "row" },
							message: "must have required property 'row'"
						};
						if (vErrors === null) vErrors = [err12];
						else vErrors.push(err12);
						errors++;
					}
					for (const key1 in data3) if (!(key1 === "column" || key1 === "row")) {
						const err13 = {
							instancePath: instancePath + "/cell",
							schemaPath: "#/$defs/Cell/additionalProperties",
							keyword: "additionalProperties",
							params: { additionalProperty: key1 },
							message: "must NOT have additional properties"
						};
						if (vErrors === null) vErrors = [err13];
						else vErrors.push(err13);
						errors++;
					}
					if (data3.column !== void 0) {
						let data4 = data3.column;
						if (!(typeof data4 == "number" && !(data4 % 1) && !isNaN(data4) && isFinite(data4))) {
							const err14 = {
								instancePath: instancePath + "/cell/column",
								schemaPath: "#/$defs/Cell/properties/column/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err14];
							else vErrors.push(err14);
							errors++;
						}
						if (typeof data4 == "number" && isFinite(data4)) {
							if (data4 > 14 || isNaN(data4)) {
								const err15 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/Cell/properties/column/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 14
									},
									message: "must be <= 14"
								};
								if (vErrors === null) vErrors = [err15];
								else vErrors.push(err15);
								errors++;
							}
							if (data4 < 0 || isNaN(data4)) {
								const err16 = {
									instancePath: instancePath + "/cell/column",
									schemaPath: "#/$defs/Cell/properties/column/minimum",
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
					if (data3.row !== void 0) {
						let data5 = data3.row;
						if (!(typeof data5 == "number" && !(data5 % 1) && !isNaN(data5) && isFinite(data5))) {
							const err17 = {
								instancePath: instancePath + "/cell/row",
								schemaPath: "#/$defs/Cell/properties/row/type",
								keyword: "type",
								params: { type: "integer" },
								message: "must be integer"
							};
							if (vErrors === null) vErrors = [err17];
							else vErrors.push(err17);
							errors++;
						}
						if (typeof data5 == "number" && isFinite(data5)) {
							if (data5 > 12 || isNaN(data5)) {
								const err18 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/Cell/properties/row/maximum",
									keyword: "maximum",
									params: {
										comparison: "<=",
										limit: 12
									},
									message: "must be <= 12"
								};
								if (vErrors === null) vErrors = [err18];
								else vErrors.push(err18);
								errors++;
							}
							if (data5 < 0 || isNaN(data5)) {
								const err19 = {
									instancePath: instancePath + "/cell/row",
									schemaPath: "#/$defs/Cell/properties/row/minimum",
									keyword: "minimum",
									params: {
										comparison: ">=",
										limit: 0
									},
									message: "must be >= 0"
								};
								if (vErrors === null) vErrors = [err19];
								else vErrors.push(err19);
								errors++;
							}
						}
					}
				} else {
					const err20 = {
						instancePath: instancePath + "/cell",
						schemaPath: "#/$defs/Cell/type",
						keyword: "type",
						params: { type: "object" },
						message: "must be object"
					};
					if (vErrors === null) vErrors = [err20];
					else vErrors.push(err20);
					errors++;
				}
			}
			if (data.kind !== void 0) {
				let data6 = data.kind;
				if (!(data6 === "bombCapacity" || data6 === "blastRange" || data6 === "movementSpeed" || data6 === "kick" || data6 === "punch" || data6 === "bracelet" || data6 === "wallPass" || data6 === "heart" || data6 === "remoteBomb" || data6 === "piercingBomb" || data6 === "powerBomb" || data6 === "landmine" || data6 === "shield" || data6 === "stopwatch" || data6 === "invisibilityCloak" || data6 === "goldenFlame" || data6 === "shortFuse" || data6 === "diarrhea" || data6 === "slowFoot" || data6 === "shortFlame" || data6 === "longFuse" || data6 === "reverseControls" || data6 === "invisibleBombs" || data6 === "oilBarrel" || data6 === "teleporter")) {
					const err21 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/$defs/PickupKind/enum",
						keyword: "enum",
						params: { allowedValues: schema143.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err21];
					else vErrors.push(err21);
					errors++;
				}
			}
		} else {
			const err22 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err22];
			else vErrors.push(err22);
			errors++;
		}
		validate118.errors = vErrors;
		return errors === 0;
	}
	var schema167 = {
		"type": "object",
		"additionalProperties": false,
		"required": [
			"type",
			"id",
			"seat",
			"source"
		],
		"properties": {
			"type": { "const": "damagePrevented" },
			"id": { "$ref": "#/$defs/EntityId" },
			"seat": { "$ref": "#/$defs/Seat" },
			"source": { "enum": ["heart", "shield"] }
		}
	};
	function validate120(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.id === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "id" },
					message: "must have required property 'id'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.seat === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "seat" },
					message: "must have required property 'seat'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.source === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "source" },
					message: "must have required property 'source'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "id" || key0 === "seat" || key0 === "source")) {
				const err4 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.type !== void 0) {
				if ("damagePrevented" !== data.type) {
					const err5 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "damagePrevented" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.id !== void 0) {
				let data1 = data.id;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err6 = {
						instancePath: instancePath + "/id",
						schemaPath: "#/$defs/EntityId/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 4294967295 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 4294967295
							},
							message: "must be <= 4294967295"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err8 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/$defs/EntityId/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
				}
			}
			if (data.seat !== void 0) {
				let data2 = data.seat;
				if (!(data2 === "one" || data2 === "two" || data2 === "three" || data2 === "four")) {
					const err9 = {
						instancePath: instancePath + "/seat",
						schemaPath: "#/$defs/Seat/enum",
						keyword: "enum",
						params: { allowedValues: schema30.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
			}
			if (data.source !== void 0) {
				let data3 = data.source;
				if (!(data3 === "heart" || data3 === "shield")) {
					const err10 = {
						instancePath: instancePath + "/source",
						schemaPath: "#/properties/source/enum",
						keyword: "enum",
						params: { allowedValues: schema167.properties.source.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
		} else {
			const err11 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err11];
			else vErrors.push(err11);
			errors++;
		}
		validate120.errors = vErrors;
		return errors === 0;
	}
	function validate106(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate107(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate107.errors : vErrors.concat(validate107.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate110(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate110.errors : vErrors.concat(validate110.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs2 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else {
			if (_valid0) {
				valid0 = true;
				passing0 = 1;
			}
			const _errs3 = errors;
			if (!validate113(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate113.errors : vErrors.concat(validate113.errors);
				errors = vErrors.length;
			}
			var _valid0 = _errs3 === errors;
			if (_valid0 && valid0) {
				valid0 = false;
				passing0 = [passing0, 2];
			} else {
				if (_valid0) {
					valid0 = true;
					passing0 = 2;
				}
				const _errs4 = errors;
				if (!validate115(data, {
					instancePath,
					parentData,
					parentDataProperty,
					rootData
				})) {
					vErrors = vErrors === null ? validate115.errors : vErrors.concat(validate115.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs4 === errors;
				if (_valid0 && valid0) {
					valid0 = false;
					passing0 = [passing0, 3];
				} else {
					if (_valid0) {
						valid0 = true;
						passing0 = 3;
					}
					const _errs5 = errors;
					if (!validate118(data, {
						instancePath,
						parentData,
						parentDataProperty,
						rootData
					})) {
						vErrors = vErrors === null ? validate118.errors : vErrors.concat(validate118.errors);
						errors = vErrors.length;
					}
					var _valid0 = _errs5 === errors;
					if (_valid0 && valid0) {
						valid0 = false;
						passing0 = [passing0, 4];
					} else {
						if (_valid0) {
							valid0 = true;
							passing0 = 4;
						}
						const _errs6 = errors;
						if (!validate120(data, {
							instancePath,
							parentData,
							parentDataProperty,
							rootData
						})) {
							vErrors = vErrors === null ? validate120.errors : vErrors.concat(validate120.errors);
							errors = vErrors.length;
						}
						var _valid0 = _errs6 === errors;
						if (_valid0 && valid0) {
							valid0 = false;
							passing0 = [passing0, 5];
						} else if (_valid0) {
							valid0 = true;
							passing0 = 5;
						}
					}
				}
			}
		}
		if (!valid0) {
			const err0 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate106.errors = vErrors;
		return errors === 0;
	}
	function validate105(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.tick === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "tick" },
					message: "must have required property 'tick'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.index === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "index" },
					message: "must have required property 'index'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.kind === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "kind" },
					message: "must have required property 'kind'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "tick" || key0 === "index" || key0 === "kind")) {
				const err3 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.tick !== void 0) {
				let data0 = data.tick;
				if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0) && isFinite(data0))) {
					const err4 = {
						instancePath: instancePath + "/tick",
						schemaPath: "#/$defs/Tick/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data0 == "number" && isFinite(data0)) {
					if (data0 > 9007199254740991 || isNaN(data0)) {
						const err5 = {
							instancePath: instancePath + "/tick",
							schemaPath: "#/$defs/Tick/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 9007199254740991
							},
							message: "must be <= 9007199254740991"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data0 < 0 || isNaN(data0)) {
						const err6 = {
							instancePath: instancePath + "/tick",
							schemaPath: "#/$defs/Tick/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
				}
			}
			if (data.index !== void 0) {
				let data1 = data.index;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err7 = {
						instancePath: instancePath + "/index",
						schemaPath: "#/properties/index/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err7];
					else vErrors.push(err7);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 191 || isNaN(data1)) {
						const err8 = {
							instancePath: instancePath + "/index",
							schemaPath: "#/properties/index/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 191
							},
							message: "must be <= 191"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err9 = {
							instancePath: instancePath + "/index",
							schemaPath: "#/properties/index/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
				}
			}
			if (data.kind !== void 0) {
				if (!validate106(data.kind, {
					instancePath: instancePath + "/kind",
					parentData: data,
					parentDataProperty: "kind",
					rootData
				})) {
					vErrors = vErrors === null ? validate106.errors : vErrors.concat(validate106.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err10 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err10];
			else vErrors.push(err10);
			errors++;
		}
		validate105.errors = vErrors;
		return errors === 0;
	}
	function validate102(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.room === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "room" },
					message: "must have required property 'room'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.state === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "state" },
					message: "must have required property 'state'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.events === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "events" },
					message: "must have required property 'events'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "room" || key0 === "state" || key0 === "events")) {
				const err4 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			if (data.type !== void 0) {
				if ("snapshot" !== data.type) {
					const err5 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "snapshot" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.room !== void 0) {
				if (!validate17(data.room, {
					instancePath: instancePath + "/room",
					parentData: data,
					parentDataProperty: "room",
					rootData
				})) {
					vErrors = vErrors === null ? validate17.errors : vErrors.concat(validate17.errors);
					errors = vErrors.length;
				}
			}
			if (data.state !== void 0) {
				if (!validate66(data.state, {
					instancePath: instancePath + "/state",
					parentData: data,
					parentDataProperty: "state",
					rootData
				})) {
					vErrors = vErrors === null ? validate66.errors : vErrors.concat(validate66.errors);
					errors = vErrors.length;
				}
			}
			if (data.events !== void 0) {
				let data3 = data.events;
				if (Array.isArray(data3)) {
					if (data3.length > 192) {
						const err6 = {
							instancePath: instancePath + "/events",
							schemaPath: "#/properties/events/maxItems",
							keyword: "maxItems",
							params: { limit: 192 },
							message: "must NOT have more than 192 items"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					if (data3.length < 0) {
						const err7 = {
							instancePath: instancePath + "/events",
							schemaPath: "#/properties/events/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					const len0 = data3.length;
					for (let i0 = 0; i0 < len0; i0++) if (!validate105(data3[i0], {
						instancePath: instancePath + "/events/" + i0,
						parentData: data3,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate105.errors : vErrors.concat(validate105.errors);
						errors = vErrors.length;
					}
				} else {
					const err8 = {
						instancePath: instancePath + "/events",
						schemaPath: "#/properties/events/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
			}
		} else {
			const err9 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err9];
			else vErrors.push(err9);
			errors++;
		}
		validate102.errors = vErrors;
		return errors === 0;
	}
	function validate125(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
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
			if (data.sequence === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "sequence" },
					message: "must have required property 'sequence'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "roomId" || key0 === "sequence")) {
				const err3 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.type !== void 0) {
				if ("leftRoom" !== data.type) {
					const err4 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "leftRoom" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.roomId !== void 0) {
				let data1 = data.roomId;
				if (typeof data1 === "string") {
					if (func3(data1) > 32) {
						const err5 = {
							instancePath: instancePath + "/roomId",
							schemaPath: "#/$defs/RoomId/maxLength",
							keyword: "maxLength",
							params: { limit: 32 },
							message: "must NOT have more than 32 characters"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (func3(data1) < 32) {
						const err6 = {
							instancePath: instancePath + "/roomId",
							schemaPath: "#/$defs/RoomId/minLength",
							keyword: "minLength",
							params: { limit: 32 },
							message: "must NOT have fewer than 32 characters"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					if (!pattern0.test(data1)) {
						const err7 = {
							instancePath: instancePath + "/roomId",
							schemaPath: "#/$defs/RoomId/pattern",
							keyword: "pattern",
							params: { pattern: "^[0-9a-f]{32}$" },
							message: "must match pattern \"^[0-9a-f]{32}$\""
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
				} else {
					const err8 = {
						instancePath: instancePath + "/roomId",
						schemaPath: "#/$defs/RoomId/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
			}
			if (data.sequence !== void 0) {
				let data2 = data.sequence;
				if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
					const err9 = {
						instancePath: instancePath + "/sequence",
						schemaPath: "#/$defs/Counter/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
				if (typeof data2 == "number" && isFinite(data2)) {
					if (data2 > 2147483647 || isNaN(data2)) {
						const err10 = {
							instancePath: instancePath + "/sequence",
							schemaPath: "#/$defs/Counter/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err10];
						else vErrors.push(err10);
						errors++;
					}
					if (data2 < 1 || isNaN(data2)) {
						const err11 = {
							instancePath: instancePath + "/sequence",
							schemaPath: "#/$defs/Counter/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
				}
			}
		} else {
			const err12 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err12];
			else vErrors.push(err12);
			errors++;
		}
		validate125.errors = vErrors;
		return errors === 0;
	}
	function validate127(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
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
			for (const key0 in data) if (!(key0 === "type" || key0 === "roomId")) {
				const err2 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.type !== void 0) {
				if ("seatReplaced" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "seatReplaced" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.roomId !== void 0) {
				let data1 = data.roomId;
				if (typeof data1 === "string") {
					if (func3(data1) > 32) {
						const err4 = {
							instancePath: instancePath + "/roomId",
							schemaPath: "#/$defs/RoomId/maxLength",
							keyword: "maxLength",
							params: { limit: 32 },
							message: "must NOT have more than 32 characters"
						};
						if (vErrors === null) vErrors = [err4];
						else vErrors.push(err4);
						errors++;
					}
					if (func3(data1) < 32) {
						const err5 = {
							instancePath: instancePath + "/roomId",
							schemaPath: "#/$defs/RoomId/minLength",
							keyword: "minLength",
							params: { limit: 32 },
							message: "must NOT have fewer than 32 characters"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (!pattern0.test(data1)) {
						const err6 = {
							instancePath: instancePath + "/roomId",
							schemaPath: "#/$defs/RoomId/pattern",
							keyword: "pattern",
							params: { pattern: "^[0-9a-f]{32}$" },
							message: "must match pattern \"^[0-9a-f]{32}$\""
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
				} else {
					const err7 = {
						instancePath: instancePath + "/roomId",
						schemaPath: "#/$defs/RoomId/type",
						keyword: "type",
						params: { type: "string" },
						message: "must be string"
					};
					if (vErrors === null) vErrors = [err7];
					else vErrors.push(err7);
					errors++;
				}
			}
		} else {
			const err8 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err8];
			else vErrors.push(err8);
			errors++;
		}
		validate127.errors = vErrors;
		return errors === 0;
	}
	var schema176 = { "enum": [
		"invalidMessage",
		"authenticationFailed",
		"connectionInUse",
		"notAuthenticated",
		"alreadyAuthenticated",
		"alreadyInRoom",
		"notInRoom",
		"roomNotFound",
		"roomFull",
		"staleRoom",
		"invalidPhase",
		"staleSequence",
		"duplicateAction",
		"rateLimited",
		"capacityReached",
		"resumeRejected",
		"staleSettings",
		"settingsLocked",
		"unsupportedConfiguration"
	] };
	function validate129(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.type === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.code === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "code" },
					message: "must have required property 'code'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.sequence === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "sequence" },
					message: "must have required property 'sequence'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "code" || key0 === "sequence")) {
				const err3 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.type !== void 0) {
				if ("error" !== data.type) {
					const err4 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "error" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.code !== void 0) {
				let data1 = data.code;
				if (!(data1 === "invalidMessage" || data1 === "authenticationFailed" || data1 === "connectionInUse" || data1 === "notAuthenticated" || data1 === "alreadyAuthenticated" || data1 === "alreadyInRoom" || data1 === "notInRoom" || data1 === "roomNotFound" || data1 === "roomFull" || data1 === "staleRoom" || data1 === "invalidPhase" || data1 === "staleSequence" || data1 === "duplicateAction" || data1 === "rateLimited" || data1 === "capacityReached" || data1 === "resumeRejected" || data1 === "staleSettings" || data1 === "settingsLocked" || data1 === "unsupportedConfiguration")) {
					const err5 = {
						instancePath: instancePath + "/code",
						schemaPath: "#/$defs/V14ErrorCode/enum",
						keyword: "enum",
						params: { allowedValues: schema176.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.sequence !== void 0) {
				let data2 = data.sequence;
				const _errs6 = errors;
				let valid2 = false;
				let passing0 = null;
				const _errs7 = errors;
				if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
					const err6 = {
						instancePath: instancePath + "/sequence",
						schemaPath: "#/$defs/Counter/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
				if (typeof data2 == "number" && isFinite(data2)) {
					if (data2 > 2147483647 || isNaN(data2)) {
						const err7 = {
							instancePath: instancePath + "/sequence",
							schemaPath: "#/$defs/Counter/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data2 < 1 || isNaN(data2)) {
						const err8 = {
							instancePath: instancePath + "/sequence",
							schemaPath: "#/$defs/Counter/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
				}
				var _valid0 = _errs7 === errors;
				if (_valid0) {
					valid2 = true;
					passing0 = 0;
				}
				const _errs10 = errors;
				if (data2 !== null) {
					const err9 = {
						instancePath: instancePath + "/sequence",
						schemaPath: "#/properties/sequence/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
				var _valid0 = _errs10 === errors;
				if (_valid0 && valid2) {
					valid2 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid2 = true;
					passing0 = 1;
				}
				if (!valid2) {
					const err10 = {
						instancePath: instancePath + "/sequence",
						schemaPath: "#/properties/sequence/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				} else {
					errors = _errs6;
					if (vErrors !== null) {
						if (_errs6) vErrors.length = _errs6;
						else vErrors = null;
					}
				}
			}
		} else {
			const err11 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err11];
			else vErrors.push(err11);
			errors++;
		}
		validate129.errors = vErrors;
		return errors === 0;
	}
	function validate132(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate12(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate12.errors : vErrors.concat(validate12.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate14(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate14.errors : vErrors.concat(validate14.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs2 === errors;
		if (_valid0 && valid0) {
			valid0 = false;
			passing0 = [passing0, 1];
		} else {
			if (_valid0) {
				valid0 = true;
				passing0 = 1;
			}
			const _errs3 = errors;
			if (!validate16(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate16.errors : vErrors.concat(validate16.errors);
				errors = vErrors.length;
			}
			var _valid0 = _errs3 === errors;
			if (_valid0 && valid0) {
				valid0 = false;
				passing0 = [passing0, 2];
			} else {
				if (_valid0) {
					valid0 = true;
					passing0 = 2;
				}
				const _errs4 = errors;
				if (!validate99(data, {
					instancePath,
					parentData,
					parentDataProperty,
					rootData
				})) {
					vErrors = vErrors === null ? validate99.errors : vErrors.concat(validate99.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs4 === errors;
				if (_valid0 && valid0) {
					valid0 = false;
					passing0 = [passing0, 3];
				} else {
					if (_valid0) {
						valid0 = true;
						passing0 = 3;
					}
					const _errs5 = errors;
					if (!validate102(data, {
						instancePath,
						parentData,
						parentDataProperty,
						rootData
					})) {
						vErrors = vErrors === null ? validate102.errors : vErrors.concat(validate102.errors);
						errors = vErrors.length;
					}
					var _valid0 = _errs5 === errors;
					if (_valid0 && valid0) {
						valid0 = false;
						passing0 = [passing0, 4];
					} else {
						if (_valid0) {
							valid0 = true;
							passing0 = 4;
						}
						const _errs6 = errors;
						if (!validate125(data, {
							instancePath,
							parentData,
							parentDataProperty,
							rootData
						})) {
							vErrors = vErrors === null ? validate125.errors : vErrors.concat(validate125.errors);
							errors = vErrors.length;
						}
						var _valid0 = _errs6 === errors;
						if (_valid0 && valid0) {
							valid0 = false;
							passing0 = [passing0, 5];
						} else {
							if (_valid0) {
								valid0 = true;
								passing0 = 5;
							}
							const _errs7 = errors;
							if (!validate127(data, {
								instancePath,
								parentData,
								parentDataProperty,
								rootData
							})) {
								vErrors = vErrors === null ? validate127.errors : vErrors.concat(validate127.errors);
								errors = vErrors.length;
							}
							var _valid0 = _errs7 === errors;
							if (_valid0 && valid0) {
								valid0 = false;
								passing0 = [passing0, 6];
							} else {
								if (_valid0) {
									valid0 = true;
									passing0 = 6;
								}
								const _errs8 = errors;
								if (!validate129(data, {
									instancePath,
									parentData,
									parentDataProperty,
									rootData
								})) {
									vErrors = vErrors === null ? validate129.errors : vErrors.concat(validate129.errors);
									errors = vErrors.length;
								}
								var _valid0 = _errs8 === errors;
								if (_valid0 && valid0) {
									valid0 = false;
									passing0 = [passing0, 7];
								} else if (_valid0) {
									valid0 = true;
									passing0 = 7;
								}
							}
						}
					}
				}
			}
		}
		if (!valid0) {
			const err0 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate132.errors = vErrors;
		return errors === 0;
	}
}));
//#endregion
export default require_gameplay_v15();
