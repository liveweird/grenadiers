//#region \0rolldown/runtime.js
var __commonJSMin = (cb, mod) => () => (mod || (cb((mod = { exports: {} }).exports, mod, mod.exports), cb = null), mod.exports);
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
//#region \0grenadiers-validator-entry/gameplay-v20.cjs
var require_gameplay_v20 = /* @__PURE__ */ __commonJSMin(((exports, module) => {
	module.exports = validate323;
	module.exports.default = validate323;
	function validate24(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (20 !== data) {
			const err0 = {
				instancePath,
				schemaPath: "#/const",
				keyword: "const",
				params: { allowedValue: 20 },
				message: "must be equal to constant"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate24.errors = vErrors;
		return errors === 0;
	}
	function validate23(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate24(data.protocolVersion, {
					instancePath: instancePath + "/protocolVersion",
					parentData: data,
					parentDataProperty: "protocolVersion",
					rootData
				})) {
					vErrors = vErrors === null ? validate24.errors : vErrors.concat(validate24.errors);
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
		validate23.errors = vErrors;
		return errors === 0;
	}
	function validate28(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(typeof data == "number" && !(data % 1) && !isNaN(data) && isFinite(data))) {
			const err0 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "integer" },
				message: "must be integer"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		if (typeof data == "number" && isFinite(data)) {
			if (data > 65535 || isNaN(data)) {
				const err1 = {
					instancePath,
					schemaPath: "#/maximum",
					keyword: "maximum",
					params: {
						comparison: "<=",
						limit: 65535
					},
					message: "must be <= 65535"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data < 0 || isNaN(data)) {
				const err2 = {
					instancePath,
					schemaPath: "#/minimum",
					keyword: "minimum",
					params: {
						comparison: ">=",
						limit: 0
					},
					message: "must be >= 0"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
		}
		validate28.errors = vErrors;
		return errors === 0;
	}
	function validate27(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate28(data.supportedVersion, {
					instancePath: instancePath + "/supportedVersion",
					parentData: data,
					parentDataProperty: "supportedVersion",
					rootData
				})) {
					vErrors = vErrors === null ? validate28.errors : vErrors.concat(validate28.errors);
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
		validate27.errors = vErrors;
		return errors === 0;
	}
	function validate32(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(typeof data == "number" && !(data % 1) && !isNaN(data) && isFinite(data))) {
			const err0 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "integer" },
				message: "must be integer"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		if (typeof data == "number" && isFinite(data)) {
			if (data > 2147483647 || isNaN(data)) {
				const err1 = {
					instancePath,
					schemaPath: "#/maximum",
					keyword: "maximum",
					params: {
						comparison: "<=",
						limit: 2147483647
					},
					message: "must be <= 2147483647"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data < 1 || isNaN(data)) {
				const err2 = {
					instancePath,
					schemaPath: "#/minimum",
					keyword: "minimum",
					params: {
						comparison: ">=",
						limit: 1
					},
					message: "must be >= 1"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
		}
		validate32.errors = vErrors;
		return errors === 0;
	}
	var schema14 = {
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
	var func2 = Object.prototype.hasOwnProperty;
	var func0 = require_equal().default;
	var func3 = require_ucs2length().default;
	var pattern0 = /* @__PURE__ */ new RegExp("^[0-9a-f]{32}$", "u");
	function validate35(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (typeof data === "string") {
			if (func3(data) > 32) {
				const err0 = {
					instancePath,
					schemaPath: "#/maxLength",
					keyword: "maxLength",
					params: { limit: 32 },
					message: "must NOT have more than 32 characters"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (func3(data) < 32) {
				const err1 = {
					instancePath,
					schemaPath: "#/minLength",
					keyword: "minLength",
					params: { limit: 32 },
					message: "must NOT have fewer than 32 characters"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (!pattern0.test(data)) {
				const err2 = {
					instancePath,
					schemaPath: "#/pattern",
					keyword: "pattern",
					params: { pattern: "^[0-9a-f]{32}$" },
					message: "must match pattern \"^[0-9a-f]{32}$\""
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
		} else {
			const err3 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate35.errors = vErrors;
		return errors === 0;
	}
	var pattern1 = /* @__PURE__ */ new RegExp("^[A-HJ-NP-Z2-9]{8}$", "u");
	function validate37(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (typeof data === "string") {
			if (func3(data) > 8) {
				const err0 = {
					instancePath,
					schemaPath: "#/maxLength",
					keyword: "maxLength",
					params: { limit: 8 },
					message: "must NOT have more than 8 characters"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (func3(data) < 8) {
				const err1 = {
					instancePath,
					schemaPath: "#/minLength",
					keyword: "minLength",
					params: { limit: 8 },
					message: "must NOT have fewer than 8 characters"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (!pattern1.test(data)) {
				const err2 = {
					instancePath,
					schemaPath: "#/pattern",
					keyword: "pattern",
					params: { pattern: "^[A-HJ-NP-Z2-9]{8}$" },
					message: "must match pattern \"^[A-HJ-NP-Z2-9]{8}$\""
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
		} else {
			const err3 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate37.errors = vErrors;
		return errors === 0;
	}
	function validate41(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(typeof data == "number" && !(data % 1) && !isNaN(data) && isFinite(data))) {
			const err0 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "integer" },
				message: "must be integer"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		if (typeof data == "number" && isFinite(data)) {
			if (data > 9007199254740991 || isNaN(data)) {
				const err1 = {
					instancePath,
					schemaPath: "#/maximum",
					keyword: "maximum",
					params: {
						comparison: "<=",
						limit: 9007199254740991
					},
					message: "must be <= 9007199254740991"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data < 0 || isNaN(data)) {
				const err2 = {
					instancePath,
					schemaPath: "#/minimum",
					keyword: "minimum",
					params: {
						comparison: ">=",
						limit: 0
					},
					message: "must be >= 0"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
		}
		validate41.errors = vErrors;
		return errors === 0;
	}
	function validate43(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (10 !== data) {
			const err0 = {
				instancePath,
				schemaPath: "#/const",
				keyword: "const",
				params: { allowedValue: 10 },
				message: "must be equal to constant"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate43.errors = vErrors;
		return errors === 0;
	}
	var schema19 = { "enum": [
		"fixed_v1",
		"generated_v1",
		"generated_v2"
	] };
	function validate45(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(data === "fixed_v1" || data === "generated_v1" || data === "generated_v2")) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema19.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate45.errors = vErrors;
		return errors === 0;
	}
	function validate47(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(typeof data == "number" && !(data % 1) && !isNaN(data) && isFinite(data))) {
			const err0 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "integer" },
				message: "must be integer"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		if (typeof data == "number" && isFinite(data)) {
			if (data > 2147483647 || isNaN(data)) {
				const err1 = {
					instancePath,
					schemaPath: "#/maximum",
					keyword: "maximum",
					params: {
						comparison: "<=",
						limit: 2147483647
					},
					message: "must be <= 2147483647"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data < 0 || isNaN(data)) {
				const err2 = {
					instancePath,
					schemaPath: "#/minimum",
					keyword: "minimum",
					params: {
						comparison: ">=",
						limit: 0
					},
					message: "must be >= 0"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
		}
		validate47.errors = vErrors;
		return errors === 0;
	}
	var schema23 = { "enum": [
		"one",
		"two",
		"three",
		"four"
	] };
	function validate51(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(data === "one" || data === "two" || data === "three" || data === "four")) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema23.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate51.errors = vErrors;
		return errors === 0;
	}
	function validate53(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(typeof data == "number" && !(data % 1) && !isNaN(data) && isFinite(data))) {
			const err0 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "integer" },
				message: "must be integer"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		if (typeof data == "number" && isFinite(data)) {
			if (data > 9 || isNaN(data)) {
				const err1 = {
					instancePath,
					schemaPath: "#/maximum",
					keyword: "maximum",
					params: {
						comparison: "<=",
						limit: 9
					},
					message: "must be <= 9"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data < 0 || isNaN(data)) {
				const err2 = {
					instancePath,
					schemaPath: "#/minimum",
					keyword: "minimum",
					params: {
						comparison: ">=",
						limit: 0
					},
					message: "must be >= 0"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
		}
		validate53.errors = vErrors;
		return errors === 0;
	}
	function validate50(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate51(data.seat, {
					instancePath: instancePath + "/seat",
					parentData: data,
					parentDataProperty: "seat",
					rootData
				})) {
					vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
					errors = vErrors.length;
				}
			}
			if (data.wins !== void 0) {
				if (!validate53(data.wins, {
					instancePath: instancePath + "/wins",
					parentData: data,
					parentDataProperty: "wins",
					rootData
				})) {
					vErrors = vErrors === null ? validate53.errors : vErrors.concat(validate53.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err3 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate50.errors = vErrors;
		return errors === 0;
	}
	function validate49(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			for (let i0 = 0; i0 < len0; i0++) if (!validate50(data[i0], {
				instancePath: instancePath + "/" + i0,
				parentData: data,
				parentDataProperty: i0,
				rootData
			})) {
				vErrors = vErrors === null ? validate50.errors : vErrors.concat(validate50.errors);
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
		validate49.errors = vErrors;
		return errors === 0;
	}
	function validate62(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate51(data.seat, {
					instancePath: instancePath + "/seat",
					parentData: data,
					parentDataProperty: "seat",
					rootData
				})) {
					vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
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
		validate62.errors = vErrors;
		return errors === 0;
	}
	var schema28 = {
		"type": "object",
		"additionalProperties": false,
		"required": ["type", "reason"],
		"properties": {
			"type": { "const": "draw" },
			"reason": { "enum": ["mutualElimination", "timeout"] }
		}
	};
	function validate65(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if ("draw" !== data.type) {
					const err3 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
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
						schemaPath: "#/properties/reason/enum",
						keyword: "enum",
						params: { allowedValues: schema28.properties.reason.enum },
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
		validate65.errors = vErrors;
		return errors === 0;
	}
	function validate61(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate62(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate62.errors : vErrors.concat(validate62.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate65(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate65.errors : vErrors.concat(validate65.errors);
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
		validate61.errors = vErrors;
		return errors === 0;
	}
	function validate57(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate32(data.roundId, {
					instancePath: instancePath + "/roundId",
					parentData: data,
					parentDataProperty: "roundId",
					rootData
				})) {
					vErrors = vErrors === null ? validate32.errors : vErrors.concat(validate32.errors);
					errors = vErrors.length;
				}
			}
			if (data.roundNumber !== void 0) {
				if (!validate32(data.roundNumber, {
					instancePath: instancePath + "/roundNumber",
					parentData: data,
					parentDataProperty: "roundNumber",
					rootData
				})) {
					vErrors = vErrors === null ? validate32.errors : vErrors.concat(validate32.errors);
					errors = vErrors.length;
				}
			}
			if (data.endedAt !== void 0) {
				if (!validate41(data.endedAt, {
					instancePath: instancePath + "/endedAt",
					parentData: data,
					parentDataProperty: "endedAt",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
			}
			if (data.result !== void 0) {
				if (!validate61(data.result, {
					instancePath: instancePath + "/result",
					parentData: data,
					parentDataProperty: "result",
					rootData
				})) {
					vErrors = vErrors === null ? validate61.errors : vErrors.concat(validate61.errors);
					errors = vErrors.length;
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
		validate57.errors = vErrors;
		return errors === 0;
	}
	function validate72(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
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
						schemaPath: "#/properties/type/const",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate72.errors = vErrors;
		return errors === 0;
	}
	var schema33 = { "enum": [
		"easy",
		"standard",
		"hard"
	] };
	function validate75(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(data === "easy" || data === "standard" || data === "hard")) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema33.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate75.errors = vErrors;
		return errors === 0;
	}
	function validate74(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate75(data.difficulty, {
					instancePath: instancePath + "/difficulty",
					parentData: data,
					parentDataProperty: "difficulty",
					rootData
				})) {
					vErrors = vErrors === null ? validate75.errors : vErrors.concat(validate75.errors);
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
		validate74.errors = vErrors;
		return errors === 0;
	}
	function validate71(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate72(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate72.errors : vErrors.concat(validate72.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate74(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate74.errors : vErrors.concat(validate74.errors);
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
		validate71.errors = vErrors;
		return errors === 0;
	}
	function validate80(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
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
						schemaPath: "#/properties/type/const",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate80.errors = vErrors;
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
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.type !== void 0) {
				if ("connected" !== data.type) {
					const err2 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "connected" },
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate82.errors = vErrors;
		return errors === 0;
	}
	function validate84(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate41(data.expiresAt, {
					instancePath: instancePath + "/expiresAt",
					parentData: data,
					parentDataProperty: "expiresAt",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
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
		validate84.errors = vErrors;
		return errors === 0;
	}
	function validate87(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.type !== void 0) {
				if ("expired" !== data.type) {
					const err2 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "expired" },
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate87.errors = vErrors;
		return errors === 0;
	}
	function validate89(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.type !== void 0) {
				if ("serverControlled" !== data.type) {
					const err2 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "serverControlled" },
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate89.errors = vErrors;
		return errors === 0;
	}
	function validate79(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate80(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate80.errors : vErrors.concat(validate80.errors);
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
		} else {
			if (_valid0) {
				valid0 = true;
				passing0 = 1;
			}
			const _errs3 = errors;
			if (!validate84(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate84.errors : vErrors.concat(validate84.errors);
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
				if (!validate87(data, {
					instancePath,
					parentData,
					parentDataProperty,
					rootData
				})) {
					vErrors = vErrors === null ? validate87.errors : vErrors.concat(validate87.errors);
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
					if (!validate89(data, {
						instancePath,
						parentData,
						parentDataProperty,
						rootData
					})) {
						vErrors = vErrors === null ? validate89.errors : vErrors.concat(validate89.errors);
						errors = vErrors.length;
					}
					var _valid0 = _errs5 === errors;
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
		validate79.errors = vErrors;
		return errors === 0;
	}
	function validate69(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate51(data.seat, {
					instancePath: instancePath + "/seat",
					parentData: data,
					parentDataProperty: "seat",
					rootData
				})) {
					vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
					errors = vErrors.length;
				}
			}
			if (data.occupied !== void 0) {
				if (typeof data.occupied !== "boolean") {
					const err7 = {
						instancePath: instancePath + "/occupied",
						schemaPath: "#/properties/occupied/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err7];
					else vErrors.push(err7);
					errors++;
				}
			}
			if (data.ready !== void 0) {
				if (typeof data.ready !== "boolean") {
					const err8 = {
						instancePath: instancePath + "/ready",
						schemaPath: "#/properties/ready/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
			}
			if (data.rematchReady !== void 0) {
				if (typeof data.rematchReady !== "boolean") {
					const err9 = {
						instancePath: instancePath + "/rematchReady",
						schemaPath: "#/properties/rematchReady/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
			}
			if (data.controller !== void 0) {
				if (!validate71(data.controller, {
					instancePath: instancePath + "/controller",
					parentData: data,
					parentDataProperty: "controller",
					rootData
				})) {
					vErrors = vErrors === null ? validate71.errors : vErrors.concat(validate71.errors);
					errors = vErrors.length;
				}
			}
			if (data.connection !== void 0) {
				if (!validate79(data.connection, {
					instancePath: instancePath + "/connection",
					parentData: data,
					parentDataProperty: "connection",
					rootData
				})) {
					vErrors = vErrors === null ? validate79.errors : vErrors.concat(validate79.errors);
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
		validate69.errors = vErrors;
		return errors === 0;
	}
	function validate94(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
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
						schemaPath: "#/properties/type/const",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate94.errors = vErrors;
		return errors === 0;
	}
	var schema42 = {
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
	function validate96(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
						params: { allowedValues: schema42.properties.kind.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.endsAt !== void 0) {
				if (!validate41(data.endsAt, {
					instancePath: instancePath + "/endsAt",
					parentData: data,
					parentDataProperty: "endsAt",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err6 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err6];
			else vErrors.push(err6);
			errors++;
		}
		validate96.errors = vErrors;
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
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.type !== void 0) {
				if ("running" !== data.type) {
					const err2 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "running" },
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate99.errors = vErrors;
		return errors === 0;
	}
	function validate101(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate41(data.holdUntil, {
					instancePath: instancePath + "/holdUntil",
					parentData: data,
					parentDataProperty: "holdUntil",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
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
		validate101.errors = vErrors;
		return errors === 0;
	}
	var schema47 = {
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
				if (!validate51(data.seat, {
					instancePath: instancePath + "/seat",
					parentData: data,
					parentDataProperty: "seat",
					rootData
				})) {
					vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
					errors = vErrors.length;
				}
			}
			if (data.reason !== void 0) {
				let data2 = data.reason;
				if (!(data2 === "roundWins" || data2 === "forfeit")) {
					const err5 = {
						instancePath: instancePath + "/reason",
						schemaPath: "#/properties/reason/enum",
						keyword: "enum",
						params: { allowedValues: schema47.properties.reason.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
		} else {
			const err6 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err6];
			else vErrors.push(err6);
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
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
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
						schemaPath: "#/properties/type/const",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate110.errors = vErrors;
		return errors === 0;
	}
	var schema50 = { "enum": [
		"connectionLost",
		"serverOverloaded",
		"serverStopped",
		"internalError"
	] };
	function validate113(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(data === "connectionLost" || data === "serverOverloaded" || data === "serverStopped" || data === "internalError")) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema50.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate113.errors = vErrors;
		return errors === 0;
	}
	function validate112(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate113(data.reason, {
					instancePath: instancePath + "/reason",
					parentData: data,
					parentDataProperty: "reason",
					rootData
				})) {
					vErrors = vErrors === null ? validate113.errors : vErrors.concat(validate113.errors);
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
		validate112.errors = vErrors;
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
			if (!validate112(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate112.errors : vErrors.concat(validate112.errors);
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
		validate106.errors = vErrors;
		return errors === 0;
	}
	function validate104(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate41(data.endedAt, {
					instancePath: instancePath + "/endedAt",
					parentData: data,
					parentDataProperty: "endedAt",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
			}
			if (data.result !== void 0) {
				if (!validate106(data.result, {
					instancePath: instancePath + "/result",
					parentData: data,
					parentDataProperty: "result",
					rootData
				})) {
					vErrors = vErrors === null ? validate106.errors : vErrors.concat(validate106.errors);
					errors = vErrors.length;
				}
			}
			if (data.rematchAt !== void 0) {
				if (!validate41(data.rematchAt, {
					instancePath: instancePath + "/rematchAt",
					parentData: data,
					parentDataProperty: "rematchAt",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err6 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err6];
			else vErrors.push(err6);
			errors++;
		}
		validate104.errors = vErrors;
		return errors === 0;
	}
	function validate119(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate41(data.endedAt, {
					instancePath: instancePath + "/endedAt",
					parentData: data,
					parentDataProperty: "endedAt",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
			}
			if (data.reason !== void 0) {
				if (!validate113(data.reason, {
					instancePath: instancePath + "/reason",
					parentData: data,
					parentDataProperty: "reason",
					rootData
				})) {
					vErrors = vErrors === null ? validate113.errors : vErrors.concat(validate113.errors);
					errors = vErrors.length;
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
		validate119.errors = vErrors;
		return errors === 0;
	}
	function validate93(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate94(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate94.errors : vErrors.concat(validate94.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate96(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate96.errors : vErrors.concat(validate96.errors);
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
			if (!validate99(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate99.errors : vErrors.concat(validate99.errors);
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
				if (!validate101(data, {
					instancePath,
					parentData,
					parentDataProperty,
					rootData
				})) {
					vErrors = vErrors === null ? validate101.errors : vErrors.concat(validate101.errors);
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
					if (!validate104(data, {
						instancePath,
						parentData,
						parentDataProperty,
						rootData
					})) {
						vErrors = vErrors === null ? validate104.errors : vErrors.concat(validate104.errors);
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
						if (!validate119(data, {
							instancePath,
							parentData,
							parentDataProperty,
							rootData
						})) {
							vErrors = vErrors === null ? validate119.errors : vErrors.concat(validate119.errors);
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
		validate93.errors = vErrors;
		return errors === 0;
	}
	var schema52 = {
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
	var schema53 = { "enum": [
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
	function validate125(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		let valid0 = false;
		for (const v0 of schema53.enum) if (func0(data, v0)) {
			valid0 = true;
			break;
		}
		if (!valid0) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema53.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate125.errors = vErrors;
		return errors === 0;
	}
	function validate128(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.enabled === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "enabled" },
					message: "must have required property 'enabled'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.spawnSeconds === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "spawnSeconds" },
					message: "must have required property 'spawnSeconds'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.maxLivingMobs === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "maxLivingMobs" },
					message: "must have required property 'maxLivingMobs'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "enabled" || key0 === "spawnSeconds" || key0 === "maxLivingMobs")) {
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
			if (data.enabled !== void 0) {
				if (typeof data.enabled !== "boolean") {
					const err4 = {
						instancePath: instancePath + "/enabled",
						schemaPath: "#/properties/enabled/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.spawnSeconds !== void 0) {
				let data1 = data.spawnSeconds;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err5 = {
						instancePath: instancePath + "/spawnSeconds",
						schemaPath: "#/properties/spawnSeconds/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 60 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/spawnSeconds",
							schemaPath: "#/properties/spawnSeconds/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 60
							},
							message: "must be <= 60"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					if (data1 < 5 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/spawnSeconds",
							schemaPath: "#/properties/spawnSeconds/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 5
							},
							message: "must be >= 5"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
				}
			}
			if (data.maxLivingMobs !== void 0) {
				if (1 !== data.maxLivingMobs) {
					const err8 = {
						instancePath: instancePath + "/maxLivingMobs",
						schemaPath: "#/properties/maxLivingMobs/const",
						keyword: "const",
						params: { allowedValue: 1 },
						message: "must be equal to constant"
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
		validate128.errors = vErrors;
		return errors === 0;
	}
	function validate130(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.enabled === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "enabled" },
					message: "must have required property 'enabled'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.warningTicks === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "warningTicks" },
					message: "must have required property 'warningTicks'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.raisedTicks === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "raisedTicks" },
					message: "must have required property 'raisedTicks'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "enabled" || key0 === "warningTicks" || key0 === "raisedTicks")) {
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
			if (data.enabled !== void 0) {
				if (typeof data.enabled !== "boolean") {
					const err4 = {
						instancePath: instancePath + "/enabled",
						schemaPath: "#/properties/enabled/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.warningTicks !== void 0) {
				let data1 = data.warningTicks;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err5 = {
						instancePath: instancePath + "/warningTicks",
						schemaPath: "#/properties/warningTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 120 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/warningTicks",
							schemaPath: "#/properties/warningTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 120
							},
							message: "must be <= 120"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					if (data1 < 6 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/warningTicks",
							schemaPath: "#/properties/warningTicks/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 6
							},
							message: "must be >= 6"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
				}
			}
			if (data.raisedTicks !== void 0) {
				let data2 = data.raisedTicks;
				if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
					const err8 = {
						instancePath: instancePath + "/raisedTicks",
						schemaPath: "#/properties/raisedTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
				if (typeof data2 == "number" && isFinite(data2)) {
					if (data2 > 600 || isNaN(data2)) {
						const err9 = {
							instancePath: instancePath + "/raisedTicks",
							schemaPath: "#/properties/raisedTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 600
							},
							message: "must be <= 600"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
					if (data2 < 60 || isNaN(data2)) {
						const err10 = {
							instancePath: instancePath + "/raisedTicks",
							schemaPath: "#/properties/raisedTicks/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 60
							},
							message: "must be >= 60"
						};
						if (vErrors === null) vErrors = [err10];
						else vErrors.push(err10);
						errors++;
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
		validate130.errors = vErrors;
		return errors === 0;
	}
	function validate127(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.stormCloud === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "stormCloud" },
					message: "must have required property 'stormCloud'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.lairs === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "lairs" },
					message: "must have required property 'lairs'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.spikePits === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "spikePits" },
					message: "must have required property 'spikePits'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "stormCloud" || key0 === "lairs" || key0 === "spikePits")) {
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
			if (data.stormCloud !== void 0) {
				if (typeof data.stormCloud !== "boolean") {
					const err4 = {
						instancePath: instancePath + "/stormCloud",
						schemaPath: "#/properties/stormCloud/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.lairs !== void 0) {
				if (!validate128(data.lairs, {
					instancePath: instancePath + "/lairs",
					parentData: data,
					parentDataProperty: "lairs",
					rootData
				})) {
					vErrors = vErrors === null ? validate128.errors : vErrors.concat(validate128.errors);
					errors = vErrors.length;
				}
			}
			if (data.spikePits !== void 0) {
				if (!validate130(data.spikePits, {
					instancePath: instancePath + "/spikePits",
					parentData: data,
					parentDataProperty: "spikePits",
					rootData
				})) {
					vErrors = vErrors === null ? validate130.errors : vErrors.concat(validate130.errors);
					errors = vErrors.length;
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
		validate127.errors = vErrors;
		return errors === 0;
	}
	function validate124(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
						params: { allowedValues: schema52.properties.turnAssistUnits.enum },
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
						params: { allowedValues: schema52.properties.bombFuseTicks.enum },
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
					for (let i0 = 0; i0 < len0; i0++) if (!validate125(data4[i0], {
						instancePath: instancePath + "/enabledItems/" + i0,
						parentData: data4,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate125.errors : vErrors.concat(validate125.errors);
						errors = vErrors.length;
					}
					let i1 = data4.length;
					let j0;
					if (i1 > 1) {
						outer0: for (; i1--;) for (j0 = i1; j0--;) if (func0(data4[i1], data4[j0])) {
							const err17 = {
								instancePath: instancePath + "/enabledItems",
								schemaPath: "#/properties/enabledItems/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i1,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i1 + " are identical)"
							};
							if (vErrors === null) vErrors = [err17];
							else vErrors.push(err17);
							errors++;
							break outer0;
						}
					}
				} else {
					const err18 = {
						instancePath: instancePath + "/enabledItems",
						schemaPath: "#/properties/enabledItems/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err18];
					else vErrors.push(err18);
					errors++;
				}
			}
			if (data.environment !== void 0) {
				if (!validate127(data.environment, {
					instancePath: instancePath + "/environment",
					parentData: data,
					parentDataProperty: "environment",
					rootData
				})) {
					vErrors = vErrors === null ? validate127.errors : vErrors.concat(validate127.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err19 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err19];
			else vErrors.push(err19);
			errors++;
		}
		validate124.errors = vErrors;
		return errors === 0;
	}
	var schema59 = { "enum": [
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
	function validate139(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const vSchema0 = schema59.enum;
		if (!(func0(data, vSchema0[0]) || func0(data, vSchema0[1]) || func0(data, vSchema0[2]))) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema59.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate139.errors = vErrors;
		return errors === 0;
	}
	function validate142(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			for (const key0 in data) if (!(key0 === "width" || key0 === "height")) {
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
			if (data.width !== void 0) {
				if (11 !== data.width) {
					const err3 = {
						instancePath: instancePath + "/width",
						schemaPath: "#/properties/width/const",
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
						schemaPath: "#/properties/height/const",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err5];
			else vErrors.push(err5);
			errors++;
		}
		validate142.errors = vErrors;
		return errors === 0;
	}
	function validate144(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			for (const key0 in data) if (!(key0 === "width" || key0 === "height")) {
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
			if (data.width !== void 0) {
				if (13 !== data.width) {
					const err3 = {
						instancePath: instancePath + "/width",
						schemaPath: "#/properties/width/const",
						keyword: "const",
						params: { allowedValue: 13 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.height !== void 0) {
				if (11 !== data.height) {
					const err4 = {
						instancePath: instancePath + "/height",
						schemaPath: "#/properties/height/const",
						keyword: "const",
						params: { allowedValue: 11 },
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
		validate144.errors = vErrors;
		return errors === 0;
	}
	function validate146(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			for (const key0 in data) if (!(key0 === "width" || key0 === "height")) {
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
			if (data.width !== void 0) {
				if (15 !== data.width) {
					const err3 = {
						instancePath: instancePath + "/width",
						schemaPath: "#/properties/width/const",
						keyword: "const",
						params: { allowedValue: 15 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
			}
			if (data.height !== void 0) {
				if (13 !== data.height) {
					const err4 = {
						instancePath: instancePath + "/height",
						schemaPath: "#/properties/height/const",
						keyword: "const",
						params: { allowedValue: 13 },
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
		validate146.errors = vErrors;
		return errors === 0;
	}
	function validate141(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate142(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate142.errors : vErrors.concat(validate142.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate144(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate144.errors : vErrors.concat(validate144.errors);
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
			if (!validate146(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate146.errors : vErrors.concat(validate146.errors);
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
		validate141.errors = vErrors;
		return errors === 0;
	}
	function validate138(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate139(data.controllers, {
					instancePath: instancePath + "/controllers",
					parentData: data,
					parentDataProperty: "controllers",
					rootData
				})) {
					vErrors = vErrors === null ? validate139.errors : vErrors.concat(validate139.errors);
					errors = vErrors.length;
				}
			}
			if (data.cpuDifficulty !== void 0) {
				if (data.cpuDifficulty !== null) {
					const err5 = {
						instancePath: instancePath + "/cpuDifficulty",
						schemaPath: "#/properties/cpuDifficulty/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.board !== void 0) {
				if (!validate141(data.board, {
					instancePath: instancePath + "/board",
					parentData: data,
					parentDataProperty: "board",
					rootData
				})) {
					vErrors = vErrors === null ? validate141.errors : vErrors.concat(validate141.errors);
					errors = vErrors.length;
				}
			}
			if (data.environment !== void 0) {
				if (!validate127(data.environment, {
					instancePath: instancePath + "/environment",
					parentData: data,
					parentDataProperty: "environment",
					rootData
				})) {
					vErrors = vErrors === null ? validate127.errors : vErrors.concat(validate127.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err6 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err6];
			else vErrors.push(err6);
			errors++;
		}
		validate138.errors = vErrors;
		return errors === 0;
	}
	var schema65 = { "enum": [
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
	function validate152(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		let valid0 = false;
		for (const v0 of schema65.enum) if (func0(data, v0)) {
			valid0 = true;
			break;
		}
		if (!valid0) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema65.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate152.errors = vErrors;
		return errors === 0;
	}
	function validate151(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate152(data.controllers, {
					instancePath: instancePath + "/controllers",
					parentData: data,
					parentDataProperty: "controllers",
					rootData
				})) {
					vErrors = vErrors === null ? validate152.errors : vErrors.concat(validate152.errors);
					errors = vErrors.length;
				}
			}
			if (data.cpuDifficulty !== void 0) {
				if (!validate75(data.cpuDifficulty, {
					instancePath: instancePath + "/cpuDifficulty",
					parentData: data,
					parentDataProperty: "cpuDifficulty",
					rootData
				})) {
					vErrors = vErrors === null ? validate75.errors : vErrors.concat(validate75.errors);
					errors = vErrors.length;
				}
			}
			if (data.board !== void 0) {
				if (!validate141(data.board, {
					instancePath: instancePath + "/board",
					parentData: data,
					parentDataProperty: "board",
					rootData
				})) {
					vErrors = vErrors === null ? validate141.errors : vErrors.concat(validate141.errors);
					errors = vErrors.length;
				}
			}
			if (data.environment !== void 0) {
				if (!validate127(data.environment, {
					instancePath: instancePath + "/environment",
					parentData: data,
					parentDataProperty: "environment",
					rootData
				})) {
					vErrors = vErrors === null ? validate127.errors : vErrors.concat(validate127.errors);
					errors = vErrors.length;
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
		validate151.errors = vErrors;
		return errors === 0;
	}
	function validate137(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate138(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate138.errors : vErrors.concat(validate138.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate151(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate151.errors : vErrors.concat(validate151.errors);
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
		validate137.errors = vErrors;
		return errors === 0;
	}
	function validate34(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			for (const prop0 of schema14.required) if (data[prop0] === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: prop0 },
					message: "must have required property '" + prop0 + "'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			for (const key0 in data) if (!func2.call(schema14.properties, key0)) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.roomId !== void 0) {
				if (!validate35(data.roomId, {
					instancePath: instancePath + "/roomId",
					parentData: data,
					parentDataProperty: "roomId",
					rootData
				})) {
					vErrors = vErrors === null ? validate35.errors : vErrors.concat(validate35.errors);
					errors = vErrors.length;
				}
			}
			if (data.roomCode !== void 0) {
				if (!validate37(data.roomCode, {
					instancePath: instancePath + "/roomCode",
					parentData: data,
					parentDataProperty: "roomCode",
					rootData
				})) {
					vErrors = vErrors === null ? validate37.errors : vErrors.concat(validate37.errors);
					errors = vErrors.length;
				}
			}
			if (data.roundId !== void 0) {
				if (!validate32(data.roundId, {
					instancePath: instancePath + "/roundId",
					parentData: data,
					parentDataProperty: "roundId",
					rootData
				})) {
					vErrors = vErrors === null ? validate32.errors : vErrors.concat(validate32.errors);
					errors = vErrors.length;
				}
			}
			if (data.revision !== void 0) {
				if (!validate32(data.revision, {
					instancePath: instancePath + "/revision",
					parentData: data,
					parentDataProperty: "revision",
					rootData
				})) {
					vErrors = vErrors === null ? validate32.errors : vErrors.concat(validate32.errors);
					errors = vErrors.length;
				}
			}
			if (data.tick !== void 0) {
				if (!validate41(data.tick, {
					instancePath: instancePath + "/tick",
					parentData: data,
					parentDataProperty: "tick",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
			}
			if (data.rulesVersion !== void 0) {
				if (!validate43(data.rulesVersion, {
					instancePath: instancePath + "/rulesVersion",
					parentData: data,
					parentDataProperty: "rulesVersion",
					rootData
				})) {
					vErrors = vErrors === null ? validate43.errors : vErrors.concat(validate43.errors);
					errors = vErrors.length;
				}
			}
			if (data.mapId !== void 0) {
				if (!validate45(data.mapId, {
					instancePath: instancePath + "/mapId",
					parentData: data,
					parentDataProperty: "mapId",
					rootData
				})) {
					vErrors = vErrors === null ? validate45.errors : vErrors.concat(validate45.errors);
					errors = vErrors.length;
				}
			}
			if (data.roundNumber !== void 0) {
				if (!validate47(data.roundNumber, {
					instancePath: instancePath + "/roundNumber",
					parentData: data,
					parentDataProperty: "roundNumber",
					rootData
				})) {
					vErrors = vErrors === null ? validate47.errors : vErrors.concat(validate47.errors);
					errors = vErrors.length;
				}
			}
			if (data.scores !== void 0) {
				if (!validate49(data.scores, {
					instancePath: instancePath + "/scores",
					parentData: data,
					parentDataProperty: "scores",
					rootData
				})) {
					vErrors = vErrors === null ? validate49.errors : vErrors.concat(validate49.errors);
					errors = vErrors.length;
				}
			}
			if (data.lastRound !== void 0) {
				let data9 = data.lastRound;
				const _errs12 = errors;
				let valid1 = false;
				let passing0 = null;
				const _errs13 = errors;
				if (!validate57(data9, {
					instancePath: instancePath + "/lastRound",
					parentData: data,
					parentDataProperty: "lastRound",
					rootData
				})) {
					vErrors = vErrors === null ? validate57.errors : vErrors.concat(validate57.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs13 === errors;
				if (_valid0) {
					valid1 = true;
					passing0 = 0;
				}
				const _errs14 = errors;
				if (data9 !== null) {
					const err2 = {
						instancePath: instancePath + "/lastRound",
						schemaPath: "#/properties/lastRound/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err2];
					else vErrors.push(err2);
					errors++;
				}
				var _valid0 = _errs14 === errors;
				if (_valid0 && valid1) {
					valid1 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid1 = true;
					passing0 = 1;
				}
				if (!valid1) {
					const err3 = {
						instancePath: instancePath + "/lastRound",
						schemaPath: "#/properties/lastRound/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				} else {
					errors = _errs12;
					if (vErrors !== null) {
						if (_errs12) vErrors.length = _errs12;
						else vErrors = null;
					}
				}
			}
			if (data.seats !== void 0) {
				let data10 = data.seats;
				if (Array.isArray(data10)) {
					if (data10.length > 4) {
						const err4 = {
							instancePath: instancePath + "/seats",
							schemaPath: "#/properties/seats/maxItems",
							keyword: "maxItems",
							params: { limit: 4 },
							message: "must NOT have more than 4 items"
						};
						if (vErrors === null) vErrors = [err4];
						else vErrors.push(err4);
						errors++;
					}
					if (data10.length < 2) {
						const err5 = {
							instancePath: instancePath + "/seats",
							schemaPath: "#/properties/seats/minItems",
							keyword: "minItems",
							params: { limit: 2 },
							message: "must NOT have fewer than 2 items"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					const len0 = data10.length;
					for (let i0 = 0; i0 < len0; i0++) if (!validate69(data10[i0], {
						instancePath: instancePath + "/seats/" + i0,
						parentData: data10,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate69.errors : vErrors.concat(validate69.errors);
						errors = vErrors.length;
					}
				} else {
					const err6 = {
						instancePath: instancePath + "/seats",
						schemaPath: "#/properties/seats/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
			}
			if (data.phase !== void 0) {
				if (!validate93(data.phase, {
					instancePath: instancePath + "/phase",
					parentData: data,
					parentDataProperty: "phase",
					rootData
				})) {
					vErrors = vErrors === null ? validate93.errors : vErrors.concat(validate93.errors);
					errors = vErrors.length;
				}
			}
			if (data.settings !== void 0) {
				if (!validate124(data.settings, {
					instancePath: instancePath + "/settings",
					parentData: data,
					parentDataProperty: "settings",
					rootData
				})) {
					vErrors = vErrors === null ? validate124.errors : vErrors.concat(validate124.errors);
					errors = vErrors.length;
				}
			}
			if (data.settingsRevision !== void 0) {
				if (!validate32(data.settingsRevision, {
					instancePath: instancePath + "/settingsRevision",
					parentData: data,
					parentDataProperty: "settingsRevision",
					rootData
				})) {
					vErrors = vErrors === null ? validate32.errors : vErrors.concat(validate32.errors);
					errors = vErrors.length;
				}
			}
			if (data.settingsEditor !== void 0) {
				let data15 = data.settingsEditor;
				const _errs23 = errors;
				let valid4 = false;
				let passing1 = null;
				const _errs24 = errors;
				if (!validate51(data15, {
					instancePath: instancePath + "/settingsEditor",
					parentData: data,
					parentDataProperty: "settingsEditor",
					rootData
				})) {
					vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
					errors = vErrors.length;
				}
				var _valid1 = _errs24 === errors;
				if (_valid1) {
					valid4 = true;
					passing1 = 0;
				}
				const _errs25 = errors;
				if (data15 !== null) {
					const err7 = {
						instancePath: instancePath + "/settingsEditor",
						schemaPath: "#/properties/settingsEditor/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err7];
					else vErrors.push(err7);
					errors++;
				}
				var _valid1 = _errs25 === errors;
				if (_valid1 && valid4) {
					valid4 = false;
					passing1 = [passing1, 1];
				} else if (_valid1) {
					valid4 = true;
					passing1 = 1;
				}
				if (!valid4) {
					const err8 = {
						instancePath: instancePath + "/settingsEditor",
						schemaPath: "#/properties/settingsEditor/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing1 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				} else {
					errors = _errs23;
					if (vErrors !== null) {
						if (_errs23) vErrors.length = _errs23;
						else vErrors = null;
					}
				}
			}
			if (data.availableItems !== void 0) {
				let data16 = data.availableItems;
				if (Array.isArray(data16)) {
					if (data16.length > 22) {
						const err9 = {
							instancePath: instancePath + "/availableItems",
							schemaPath: "#/properties/availableItems/maxItems",
							keyword: "maxItems",
							params: { limit: 22 },
							message: "must NOT have more than 22 items"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
					if (data16.length < 0) {
						const err10 = {
							instancePath: instancePath + "/availableItems",
							schemaPath: "#/properties/availableItems/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err10];
						else vErrors.push(err10);
						errors++;
					}
					const len1 = data16.length;
					for (let i1 = 0; i1 < len1; i1++) if (!validate125(data16[i1], {
						instancePath: instancePath + "/availableItems/" + i1,
						parentData: data16,
						parentDataProperty: i1,
						rootData
					})) {
						vErrors = vErrors === null ? validate125.errors : vErrors.concat(validate125.errors);
						errors = vErrors.length;
					}
					let i2 = data16.length;
					let j0;
					if (i2 > 1) {
						outer0: for (; i2--;) for (j0 = i2; j0--;) if (func0(data16[i2], data16[j0])) {
							const err11 = {
								instancePath: instancePath + "/availableItems",
								schemaPath: "#/properties/availableItems/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i2,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i2 + " are identical)"
							};
							if (vErrors === null) vErrors = [err11];
							else vErrors.push(err11);
							errors++;
							break outer0;
						}
					}
				} else {
					const err12 = {
						instancePath: instancePath + "/availableItems",
						schemaPath: "#/properties/availableItems/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				}
			}
			if (data.configuration !== void 0) {
				if (!validate137(data.configuration, {
					instancePath: instancePath + "/configuration",
					parentData: data,
					parentDataProperty: "configuration",
					rootData
				})) {
					vErrors = vErrors === null ? validate137.errors : vErrors.concat(validate137.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err13 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err13];
			else vErrors.push(err13);
			errors++;
		}
		validate34.errors = vErrors;
		return errors === 0;
	}
	var pattern2 = /* @__PURE__ */ new RegExp("^[0-9a-f]{64}$", "u");
	function validate161(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (typeof data === "string") {
			if (func3(data) > 64) {
				const err0 = {
					instancePath,
					schemaPath: "#/maxLength",
					keyword: "maxLength",
					params: { limit: 64 },
					message: "must NOT have more than 64 characters"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (func3(data) < 64) {
				const err1 = {
					instancePath,
					schemaPath: "#/minLength",
					keyword: "minLength",
					params: { limit: 64 },
					message: "must NOT have fewer than 64 characters"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (!pattern2.test(data)) {
				const err2 = {
					instancePath,
					schemaPath: "#/pattern",
					keyword: "pattern",
					params: { pattern: "^[0-9a-f]{64}$" },
					message: "must match pattern \"^[0-9a-f]{64}$\""
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
		} else {
			const err3 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "string" },
				message: "must be string"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate161.errors = vErrors;
		return errors === 0;
	}
	var schema67 = {
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
			"stormCloud",
			"lairs",
			"mobs",
			"spikePits"
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
				"maxItems": 195,
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
			"stormCloud": { "oneOf": [{ "$ref": "#/$defs/StormCloudView" }, { "type": "null" }] },
			"lairs": {
				"type": "array",
				"minItems": 0,
				"maxItems": 2,
				"items": { "$ref": "#/$defs/LairView" }
			},
			"mobs": {
				"type": "array",
				"minItems": 0,
				"maxItems": 2,
				"items": { "$ref": "#/$defs/MobView" }
			},
			"spikePits": {
				"type": "array",
				"minItems": 0,
				"maxItems": 4,
				"items": { "$ref": "#/$defs/SpikePitView" },
				"uniqueItems": true
			}
		}
	};
	var schema70 = { "enum": ["floor", "wall"] };
	function validate167(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(data === "floor" || data === "wall")) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema70.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate167.errors = vErrors;
		return errors === 0;
	}
	function validate169(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.column === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "column" },
					message: "must have required property 'column'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.row === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "row" },
					message: "must have required property 'row'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "column" || key0 === "row")) {
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
			if (data.column !== void 0) {
				let data0 = data.column;
				if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0) && isFinite(data0))) {
					const err3 = {
						instancePath: instancePath + "/column",
						schemaPath: "#/properties/column/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
				if (typeof data0 == "number" && isFinite(data0)) {
					if (data0 > 14 || isNaN(data0)) {
						const err4 = {
							instancePath: instancePath + "/column",
							schemaPath: "#/properties/column/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 14
							},
							message: "must be <= 14"
						};
						if (vErrors === null) vErrors = [err4];
						else vErrors.push(err4);
						errors++;
					}
					if (data0 < 0 || isNaN(data0)) {
						const err5 = {
							instancePath: instancePath + "/column",
							schemaPath: "#/properties/column/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
				}
			}
			if (data.row !== void 0) {
				let data1 = data.row;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err6 = {
						instancePath: instancePath + "/row",
						schemaPath: "#/properties/row/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 12 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/row",
							schemaPath: "#/properties/row/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 12
							},
							message: "must be <= 12"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err8 = {
							instancePath: instancePath + "/row",
							schemaPath: "#/properties/row/minimum",
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
		validate169.errors = vErrors;
		return errors === 0;
	}
	function validate166(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
					for (let i0 = 0; i0 < len0; i0++) if (!validate167(data2[i0], {
						instancePath: instancePath + "/terrain/" + i0,
						parentData: data2,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate167.errors : vErrors.concat(validate167.errors);
						errors = vErrors.length;
					}
				} else {
					const err9 = {
						instancePath: instancePath + "/terrain",
						schemaPath: "#/properties/terrain/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
			}
			if (data.blocks !== void 0) {
				let data4 = data.blocks;
				if (Array.isArray(data4)) {
					if (data4.length > 99) {
						const err10 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/maxItems",
							keyword: "maxItems",
							params: { limit: 99 },
							message: "must NOT have more than 99 items"
						};
						if (vErrors === null) vErrors = [err10];
						else vErrors.push(err10);
						errors++;
					}
					if (data4.length < 0) {
						const err11 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
					const len1 = data4.length;
					for (let i1 = 0; i1 < len1; i1++) if (!validate169(data4[i1], {
						instancePath: instancePath + "/blocks/" + i1,
						parentData: data4,
						parentDataProperty: i1,
						rootData
					})) {
						vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
						errors = vErrors.length;
					}
					let i2 = data4.length;
					let j0;
					if (i2 > 1) {
						outer0: for (; i2--;) for (j0 = i2; j0--;) if (func0(data4[i2], data4[j0])) {
							const err12 = {
								instancePath: instancePath + "/blocks",
								schemaPath: "#/properties/blocks/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i2,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i2 + " are identical)"
							};
							if (vErrors === null) vErrors = [err12];
							else vErrors.push(err12);
							errors++;
							break outer0;
						}
					}
				} else {
					const err13 = {
						instancePath: instancePath + "/blocks",
						schemaPath: "#/properties/blocks/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
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
		validate166.errors = vErrors;
		return errors === 0;
	}
	function validate172(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
					for (let i0 = 0; i0 < len0; i0++) if (!validate167(data2[i0], {
						instancePath: instancePath + "/terrain/" + i0,
						parentData: data2,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate167.errors : vErrors.concat(validate167.errors);
						errors = vErrors.length;
					}
				} else {
					const err9 = {
						instancePath: instancePath + "/terrain",
						schemaPath: "#/properties/terrain/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
			}
			if (data.blocks !== void 0) {
				let data4 = data.blocks;
				if (Array.isArray(data4)) {
					if (data4.length > 143) {
						const err10 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/maxItems",
							keyword: "maxItems",
							params: { limit: 143 },
							message: "must NOT have more than 143 items"
						};
						if (vErrors === null) vErrors = [err10];
						else vErrors.push(err10);
						errors++;
					}
					if (data4.length < 0) {
						const err11 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
					const len1 = data4.length;
					for (let i1 = 0; i1 < len1; i1++) if (!validate169(data4[i1], {
						instancePath: instancePath + "/blocks/" + i1,
						parentData: data4,
						parentDataProperty: i1,
						rootData
					})) {
						vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
						errors = vErrors.length;
					}
					let i2 = data4.length;
					let j0;
					if (i2 > 1) {
						outer0: for (; i2--;) for (j0 = i2; j0--;) if (func0(data4[i2], data4[j0])) {
							const err12 = {
								instancePath: instancePath + "/blocks",
								schemaPath: "#/properties/blocks/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i2,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i2 + " are identical)"
							};
							if (vErrors === null) vErrors = [err12];
							else vErrors.push(err12);
							errors++;
							break outer0;
						}
					}
				} else {
					const err13 = {
						instancePath: instancePath + "/blocks",
						schemaPath: "#/properties/blocks/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
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
		validate172.errors = vErrors;
		return errors === 0;
	}
	function validate176(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
					for (let i0 = 0; i0 < len0; i0++) if (!validate167(data2[i0], {
						instancePath: instancePath + "/terrain/" + i0,
						parentData: data2,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate167.errors : vErrors.concat(validate167.errors);
						errors = vErrors.length;
					}
				} else {
					const err9 = {
						instancePath: instancePath + "/terrain",
						schemaPath: "#/properties/terrain/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
			}
			if (data.blocks !== void 0) {
				let data4 = data.blocks;
				if (Array.isArray(data4)) {
					if (data4.length > 195) {
						const err10 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/maxItems",
							keyword: "maxItems",
							params: { limit: 195 },
							message: "must NOT have more than 195 items"
						};
						if (vErrors === null) vErrors = [err10];
						else vErrors.push(err10);
						errors++;
					}
					if (data4.length < 0) {
						const err11 = {
							instancePath: instancePath + "/blocks",
							schemaPath: "#/properties/blocks/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
					const len1 = data4.length;
					for (let i1 = 0; i1 < len1; i1++) if (!validate169(data4[i1], {
						instancePath: instancePath + "/blocks/" + i1,
						parentData: data4,
						parentDataProperty: i1,
						rootData
					})) {
						vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
						errors = vErrors.length;
					}
					let i2 = data4.length;
					let j0;
					if (i2 > 1) {
						outer0: for (; i2--;) for (j0 = i2; j0--;) if (func0(data4[i2], data4[j0])) {
							const err12 = {
								instancePath: instancePath + "/blocks",
								schemaPath: "#/properties/blocks/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i2,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i2 + " are identical)"
							};
							if (vErrors === null) vErrors = [err12];
							else vErrors.push(err12);
							errors++;
							break outer0;
						}
					}
				} else {
					const err13 = {
						instancePath: instancePath + "/blocks",
						schemaPath: "#/properties/blocks/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
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
		validate176.errors = vErrors;
		return errors === 0;
	}
	function validate165(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate166(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate166.errors : vErrors.concat(validate166.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate172(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate172.errors : vErrors.concat(validate172.errors);
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
			if (!validate176(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate176.errors : vErrors.concat(validate176.errors);
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
		validate165.errors = vErrors;
		return errors === 0;
	}
	var schema75 = {
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
	function validate183(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(typeof data == "number" && !(data % 1) && !isNaN(data) && isFinite(data))) {
			const err0 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "integer" },
				message: "must be integer"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		if (typeof data == "number" && isFinite(data)) {
			if (data > 4294967295 || isNaN(data)) {
				const err1 = {
					instancePath,
					schemaPath: "#/maximum",
					keyword: "maximum",
					params: {
						comparison: "<=",
						limit: 4294967295
					},
					message: "must be <= 4294967295"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data < 0 || isNaN(data)) {
				const err2 = {
					instancePath,
					schemaPath: "#/minimum",
					keyword: "minimum",
					params: {
						comparison: ">=",
						limit: 0
					},
					message: "must be >= 0"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
		}
		validate183.errors = vErrors;
		return errors === 0;
	}
	function validate186(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.x === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "x" },
					message: "must have required property 'x'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.y === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "y" },
					message: "must have required property 'y'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "x" || key0 === "y")) {
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
			if (data.x !== void 0) {
				let data0 = data.x;
				if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0) && isFinite(data0))) {
					const err3 = {
						instancePath: instancePath + "/x",
						schemaPath: "#/properties/x/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
				if (typeof data0 == "number" && isFinite(data0)) {
					if (data0 > 15359 || isNaN(data0)) {
						const err4 = {
							instancePath: instancePath + "/x",
							schemaPath: "#/properties/x/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 15359
							},
							message: "must be <= 15359"
						};
						if (vErrors === null) vErrors = [err4];
						else vErrors.push(err4);
						errors++;
					}
					if (data0 < 0 || isNaN(data0)) {
						const err5 = {
							instancePath: instancePath + "/x",
							schemaPath: "#/properties/x/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
				}
			}
			if (data.y !== void 0) {
				let data1 = data.y;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err6 = {
						instancePath: instancePath + "/y",
						schemaPath: "#/properties/y/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 13311 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/y",
							schemaPath: "#/properties/y/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 13311
							},
							message: "must be <= 13311"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err8 = {
							instancePath: instancePath + "/y",
							schemaPath: "#/properties/y/minimum",
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
		validate186.errors = vErrors;
		return errors === 0;
	}
	var schema78 = { "enum": [
		"north",
		"east",
		"south",
		"west"
	] };
	function validate188(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(data === "north" || data === "east" || data === "south" || data === "west")) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema78.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate188.errors = vErrors;
		return errors === 0;
	}
	var schema80 = { "enum": [
		"kick",
		"punch",
		"bracelet",
		"wallPass",
		"heart"
	] };
	function validate191(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(data === "kick" || data === "punch" || data === "bracelet" || data === "wallPass" || data === "heart")) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema80.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate191.errors = vErrors;
		return errors === 0;
	}
	var schema81 = { "enum": [
		"remoteBomb",
		"piercingBomb",
		"powerBomb",
		"landmine"
	] };
	function validate193(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(data === "remoteBomb" || data === "piercingBomb" || data === "powerBomb" || data === "landmine")) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema81.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate193.errors = vErrors;
		return errors === 0;
	}
	function validate190(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate191(data0, {
					instancePath: instancePath + "/equipment",
					parentData: data,
					parentDataProperty: "equipment",
					rootData
				})) {
					vErrors = vErrors === null ? validate191.errors : vErrors.concat(validate191.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs4 === errors;
				if (_valid0) {
					valid1 = true;
					passing0 = 0;
				}
				const _errs5 = errors;
				if (data0 !== null) {
					const err3 = {
						instancePath: instancePath + "/equipment",
						schemaPath: "#/properties/equipment/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
				var _valid0 = _errs5 === errors;
				if (_valid0 && valid1) {
					valid1 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid1 = true;
					passing0 = 1;
				}
				if (!valid1) {
					const err4 = {
						instancePath: instancePath + "/equipment",
						schemaPath: "#/properties/equipment/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
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
				const _errs8 = errors;
				let valid2 = false;
				let passing1 = null;
				const _errs9 = errors;
				if (!validate193(data1, {
					instancePath: instancePath + "/weapon",
					parentData: data,
					parentDataProperty: "weapon",
					rootData
				})) {
					vErrors = vErrors === null ? validate193.errors : vErrors.concat(validate193.errors);
					errors = vErrors.length;
				}
				var _valid1 = _errs9 === errors;
				if (_valid1) {
					valid2 = true;
					passing1 = 0;
				}
				const _errs10 = errors;
				if (data1 !== null) {
					const err5 = {
						instancePath: instancePath + "/weapon",
						schemaPath: "#/properties/weapon/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
				var _valid1 = _errs10 === errors;
				if (_valid1 && valid2) {
					valid2 = false;
					passing1 = [passing1, 1];
				} else if (_valid1) {
					valid2 = true;
					passing1 = 1;
				}
				if (!valid2) {
					const err6 = {
						instancePath: instancePath + "/weapon",
						schemaPath: "#/properties/weapon/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing1 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				} else {
					errors = _errs8;
					if (vErrors !== null) {
						if (_errs8) vErrors.length = _errs8;
						else vErrors = null;
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
		validate190.errors = vErrors;
		return errors === 0;
	}
	function validate197(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
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
					schemaPath: "#/required",
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
					schemaPath: "#/additionalProperties",
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
						schemaPath: "#/properties/kind/const",
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
						schemaPath: "#/properties/remainingTicks/type",
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
							schemaPath: "#/properties/remainingTicks/maximum",
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
							schemaPath: "#/properties/remainingTicks/minimum",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate197.errors = vErrors;
		return errors === 0;
	}
	function validate199(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
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
					schemaPath: "#/required",
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
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("stopwatch" !== data.kind) {
					const err3 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "stopwatch" },
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
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 180 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 180
							},
							message: "must be <= 180"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate199.errors = vErrors;
		return errors === 0;
	}
	function validate201(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
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
					schemaPath: "#/required",
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
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("invisibilityCloak" !== data.kind) {
					const err3 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "invisibilityCloak" },
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
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 600 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 600
							},
							message: "must be <= 600"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate201.errors = vErrors;
		return errors === 0;
	}
	function validate203(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
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
					schemaPath: "#/required",
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
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("goldenFlame" !== data.kind) {
					const err3 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "goldenFlame" },
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
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 600 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 600
							},
							message: "must be <= 600"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate203.errors = vErrors;
		return errors === 0;
	}
	function validate205(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
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
					schemaPath: "#/required",
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
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("shortFuse" !== data.kind) {
					const err3 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "shortFuse" },
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
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 600 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 600
							},
							message: "must be <= 600"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate205.errors = vErrors;
		return errors === 0;
	}
	function validate207(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
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
					schemaPath: "#/required",
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
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("diarrhea" !== data.kind) {
					const err3 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "diarrhea" },
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
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 600 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 600
							},
							message: "must be <= 600"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate207.errors = vErrors;
		return errors === 0;
	}
	function validate209(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
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
					schemaPath: "#/required",
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
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("slowFoot" !== data.kind) {
					const err3 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "slowFoot" },
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
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 600 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 600
							},
							message: "must be <= 600"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate209.errors = vErrors;
		return errors === 0;
	}
	function validate211(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
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
					schemaPath: "#/required",
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
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("shortFlame" !== data.kind) {
					const err3 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "shortFlame" },
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
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 600 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 600
							},
							message: "must be <= 600"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate211.errors = vErrors;
		return errors === 0;
	}
	function validate213(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
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
					schemaPath: "#/required",
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
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("longFuse" !== data.kind) {
					const err3 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "longFuse" },
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
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 600 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 600
							},
							message: "must be <= 600"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate213.errors = vErrors;
		return errors === 0;
	}
	function validate215(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
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
					schemaPath: "#/required",
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
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("reverseControls" !== data.kind) {
					const err3 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "reverseControls" },
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
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 600 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 600
							},
							message: "must be <= 600"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate215.errors = vErrors;
		return errors === 0;
	}
	function validate217(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.kind === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
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
					schemaPath: "#/required",
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
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.kind !== void 0) {
				if ("invisibleBombs" !== data.kind) {
					const err3 = {
						instancePath: instancePath + "/kind",
						schemaPath: "#/properties/kind/const",
						keyword: "const",
						params: { allowedValue: "invisibleBombs" },
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
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 600 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 600
							},
							message: "must be <= 600"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 1 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err7];
			else vErrors.push(err7);
			errors++;
		}
		validate217.errors = vErrors;
		return errors === 0;
	}
	function validate196(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate197(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate197.errors : vErrors.concat(validate197.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate199(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate199.errors : vErrors.concat(validate199.errors);
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
			if (!validate201(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate201.errors : vErrors.concat(validate201.errors);
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
				if (!validate203(data, {
					instancePath,
					parentData,
					parentDataProperty,
					rootData
				})) {
					vErrors = vErrors === null ? validate203.errors : vErrors.concat(validate203.errors);
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
					if (!validate205(data, {
						instancePath,
						parentData,
						parentDataProperty,
						rootData
					})) {
						vErrors = vErrors === null ? validate205.errors : vErrors.concat(validate205.errors);
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
						if (!validate207(data, {
							instancePath,
							parentData,
							parentDataProperty,
							rootData
						})) {
							vErrors = vErrors === null ? validate207.errors : vErrors.concat(validate207.errors);
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
							if (!validate209(data, {
								instancePath,
								parentData,
								parentDataProperty,
								rootData
							})) {
								vErrors = vErrors === null ? validate209.errors : vErrors.concat(validate209.errors);
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
								if (!validate211(data, {
									instancePath,
									parentData,
									parentDataProperty,
									rootData
								})) {
									vErrors = vErrors === null ? validate211.errors : vErrors.concat(validate211.errors);
									errors = vErrors.length;
								}
								var _valid0 = _errs8 === errors;
								if (_valid0 && valid0) {
									valid0 = false;
									passing0 = [passing0, 7];
								} else {
									if (_valid0) {
										valid0 = true;
										passing0 = 7;
									}
									const _errs9 = errors;
									if (!validate213(data, {
										instancePath,
										parentData,
										parentDataProperty,
										rootData
									})) {
										vErrors = vErrors === null ? validate213.errors : vErrors.concat(validate213.errors);
										errors = vErrors.length;
									}
									var _valid0 = _errs9 === errors;
									if (_valid0 && valid0) {
										valid0 = false;
										passing0 = [passing0, 8];
									} else {
										if (_valid0) {
											valid0 = true;
											passing0 = 8;
										}
										const _errs10 = errors;
										if (!validate215(data, {
											instancePath,
											parentData,
											parentDataProperty,
											rootData
										})) {
											vErrors = vErrors === null ? validate215.errors : vErrors.concat(validate215.errors);
											errors = vErrors.length;
										}
										var _valid0 = _errs10 === errors;
										if (_valid0 && valid0) {
											valid0 = false;
											passing0 = [passing0, 9];
										} else {
											if (_valid0) {
												valid0 = true;
												passing0 = 9;
											}
											const _errs11 = errors;
											if (!validate217(data, {
												instancePath,
												parentData,
												parentDataProperty,
												rootData
											})) {
												vErrors = vErrors === null ? validate217.errors : vErrors.concat(validate217.errors);
												errors = vErrors.length;
											}
											var _valid0 = _errs11 === errors;
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
		validate196.errors = vErrors;
		return errors === 0;
	}
	function validate182(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			for (const prop0 of schema75.required) if (data[prop0] === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: prop0 },
					message: "must have required property '" + prop0 + "'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			for (const key0 in data) if (!func2.call(schema75.properties, key0)) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.type !== void 0) {
				if ("visible" !== data.type) {
					const err2 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "visible" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err2];
					else vErrors.push(err2);
					errors++;
				}
			}
			if (data.id !== void 0) {
				if (!validate183(data.id, {
					instancePath: instancePath + "/id",
					parentData: data,
					parentDataProperty: "id",
					rootData
				})) {
					vErrors = vErrors === null ? validate183.errors : vErrors.concat(validate183.errors);
					errors = vErrors.length;
				}
			}
			if (data.seat !== void 0) {
				if (!validate51(data.seat, {
					instancePath: instancePath + "/seat",
					parentData: data,
					parentDataProperty: "seat",
					rootData
				})) {
					vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
					errors = vErrors.length;
				}
			}
			if (data.position !== void 0) {
				if (!validate186(data.position, {
					instancePath: instancePath + "/position",
					parentData: data,
					parentDataProperty: "position",
					rootData
				})) {
					vErrors = vErrors === null ? validate186.errors : vErrors.concat(validate186.errors);
					errors = vErrors.length;
				}
			}
			if (data.heading !== void 0) {
				if (!validate188(data.heading, {
					instancePath: instancePath + "/heading",
					parentData: data,
					parentDataProperty: "heading",
					rootData
				})) {
					vErrors = vErrors === null ? validate188.errors : vErrors.concat(validate188.errors);
					errors = vErrors.length;
				}
			}
			if (data.speedLevel !== void 0) {
				let data5 = data.speedLevel;
				if (!(typeof data5 == "number" && !(data5 % 1) && !isNaN(data5) && isFinite(data5))) {
					const err3 = {
						instancePath: instancePath + "/speedLevel",
						schemaPath: "#/properties/speedLevel/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err3];
					else vErrors.push(err3);
					errors++;
				}
				if (typeof data5 == "number" && isFinite(data5)) {
					if (data5 > 4 || isNaN(data5)) {
						const err4 = {
							instancePath: instancePath + "/speedLevel",
							schemaPath: "#/properties/speedLevel/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 4
							},
							message: "must be <= 4"
						};
						if (vErrors === null) vErrors = [err4];
						else vErrors.push(err4);
						errors++;
					}
					if (data5 < 0 || isNaN(data5)) {
						const err5 = {
							instancePath: instancePath + "/speedLevel",
							schemaPath: "#/properties/speedLevel/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
				}
			}
			if (data.alive !== void 0) {
				if (typeof data.alive !== "boolean") {
					const err6 = {
						instancePath: instancePath + "/alive",
						schemaPath: "#/properties/alive/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
			}
			if (data.bombCapacity !== void 0) {
				let data7 = data.bombCapacity;
				if (!(typeof data7 == "number" && !(data7 % 1) && !isNaN(data7) && isFinite(data7))) {
					const err7 = {
						instancePath: instancePath + "/bombCapacity",
						schemaPath: "#/properties/bombCapacity/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err7];
					else vErrors.push(err7);
					errors++;
				}
				if (typeof data7 == "number" && isFinite(data7)) {
					if (data7 > 8 || isNaN(data7)) {
						const err8 = {
							instancePath: instancePath + "/bombCapacity",
							schemaPath: "#/properties/bombCapacity/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 8
							},
							message: "must be <= 8"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
					if (data7 < 1 || isNaN(data7)) {
						const err9 = {
							instancePath: instancePath + "/bombCapacity",
							schemaPath: "#/properties/bombCapacity/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
				}
			}
			if (data.bombRange !== void 0) {
				let data8 = data.bombRange;
				if (!(typeof data8 == "number" && !(data8 % 1) && !isNaN(data8) && isFinite(data8))) {
					const err10 = {
						instancePath: instancePath + "/bombRange",
						schemaPath: "#/properties/bombRange/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
				if (typeof data8 == "number" && isFinite(data8)) {
					if (data8 > 8 || isNaN(data8)) {
						const err11 = {
							instancePath: instancePath + "/bombRange",
							schemaPath: "#/properties/bombRange/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 8
							},
							message: "must be <= 8"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
					if (data8 < 1 || isNaN(data8)) {
						const err12 = {
							instancePath: instancePath + "/bombRange",
							schemaPath: "#/properties/bombRange/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 1
							},
							message: "must be >= 1"
						};
						if (vErrors === null) vErrors = [err12];
						else vErrors.push(err12);
						errors++;
					}
				}
			}
			if (data.liveBombCount !== void 0) {
				let data9 = data.liveBombCount;
				if (!(typeof data9 == "number" && !(data9 % 1) && !isNaN(data9) && isFinite(data9))) {
					const err13 = {
						instancePath: instancePath + "/liveBombCount",
						schemaPath: "#/properties/liveBombCount/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err13];
					else vErrors.push(err13);
					errors++;
				}
				if (typeof data9 == "number" && isFinite(data9)) {
					if (data9 > 8 || isNaN(data9)) {
						const err14 = {
							instancePath: instancePath + "/liveBombCount",
							schemaPath: "#/properties/liveBombCount/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 8
							},
							message: "must be <= 8"
						};
						if (vErrors === null) vErrors = [err14];
						else vErrors.push(err14);
						errors++;
					}
					if (data9 < 0 || isNaN(data9)) {
						const err15 = {
							instancePath: instancePath + "/liveBombCount",
							schemaPath: "#/properties/liveBombCount/minimum",
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
			if (data.inventory !== void 0) {
				if (!validate190(data.inventory, {
					instancePath: instancePath + "/inventory",
					parentData: data,
					parentDataProperty: "inventory",
					rootData
				})) {
					vErrors = vErrors === null ? validate190.errors : vErrors.concat(validate190.errors);
					errors = vErrors.length;
				}
			}
			if (data.effects !== void 0) {
				let data11 = data.effects;
				if (Array.isArray(data11)) {
					if (data11.length > 16) {
						const err16 = {
							instancePath: instancePath + "/effects",
							schemaPath: "#/properties/effects/maxItems",
							keyword: "maxItems",
							params: { limit: 16 },
							message: "must NOT have more than 16 items"
						};
						if (vErrors === null) vErrors = [err16];
						else vErrors.push(err16);
						errors++;
					}
					if (data11.length < 0) {
						const err17 = {
							instancePath: instancePath + "/effects",
							schemaPath: "#/properties/effects/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err17];
						else vErrors.push(err17);
						errors++;
					}
					const len0 = data11.length;
					for (let i0 = 0; i0 < len0; i0++) if (!validate196(data11[i0], {
						instancePath: instancePath + "/effects/" + i0,
						parentData: data11,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate196.errors : vErrors.concat(validate196.errors);
						errors = vErrors.length;
					}
				} else {
					const err18 = {
						instancePath: instancePath + "/effects",
						schemaPath: "#/properties/effects/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err18];
					else vErrors.push(err18);
					errors++;
				}
			}
			if (data.damageGraceRemainingTicks !== void 0) {
				let data13 = data.damageGraceRemainingTicks;
				if (!(typeof data13 == "number" && !(data13 % 1) && !isNaN(data13) && isFinite(data13))) {
					const err19 = {
						instancePath: instancePath + "/damageGraceRemainingTicks",
						schemaPath: "#/properties/damageGraceRemainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err19];
					else vErrors.push(err19);
					errors++;
				}
				if (typeof data13 == "number" && isFinite(data13)) {
					if (data13 > 60 || isNaN(data13)) {
						const err20 = {
							instancePath: instancePath + "/damageGraceRemainingTicks",
							schemaPath: "#/properties/damageGraceRemainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 60
							},
							message: "must be <= 60"
						};
						if (vErrors === null) vErrors = [err20];
						else vErrors.push(err20);
						errors++;
					}
					if (data13 < 0 || isNaN(data13)) {
						const err21 = {
							instancePath: instancePath + "/damageGraceRemainingTicks",
							schemaPath: "#/properties/damageGraceRemainingTicks/minimum",
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
			if (data.voluntaryActionsFrozen !== void 0) {
				if (typeof data.voluntaryActionsFrozen !== "boolean") {
					const err22 = {
						instancePath: instancePath + "/voluntaryActionsFrozen",
						schemaPath: "#/properties/voluntaryActionsFrozen/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
					};
					if (vErrors === null) vErrors = [err22];
					else vErrors.push(err22);
					errors++;
				}
			}
			if (data.lastTeleportTick !== void 0) {
				let data15 = data.lastTeleportTick;
				const _errs26 = errors;
				let valid3 = false;
				let passing0 = null;
				const _errs27 = errors;
				if (!validate41(data15, {
					instancePath: instancePath + "/lastTeleportTick",
					parentData: data,
					parentDataProperty: "lastTeleportTick",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs27 === errors;
				if (_valid0) {
					valid3 = true;
					passing0 = 0;
				}
				const _errs28 = errors;
				if (data15 !== null) {
					const err23 = {
						instancePath: instancePath + "/lastTeleportTick",
						schemaPath: "#/properties/lastTeleportTick/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err23];
					else vErrors.push(err23);
					errors++;
				}
				var _valid0 = _errs28 === errors;
				if (_valid0 && valid3) {
					valid3 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid3 = true;
					passing0 = 1;
				}
				if (!valid3) {
					const err24 = {
						instancePath: instancePath + "/lastTeleportTick",
						schemaPath: "#/properties/lastTeleportTick/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err24];
					else vErrors.push(err24);
					errors++;
				} else {
					errors = _errs26;
					if (vErrors !== null) {
						if (_errs26) vErrors.length = _errs26;
						else vErrors = null;
					}
				}
			}
			if (data.stormSlowRemainingTicks !== void 0) {
				let data16 = data.stormSlowRemainingTicks;
				if (!(typeof data16 == "number" && !(data16 % 1) && !isNaN(data16) && isFinite(data16))) {
					const err25 = {
						instancePath: instancePath + "/stormSlowRemainingTicks",
						schemaPath: "#/properties/stormSlowRemainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err25];
					else vErrors.push(err25);
					errors++;
				}
				if (typeof data16 == "number" && isFinite(data16)) {
					if (data16 > 180 || isNaN(data16)) {
						const err26 = {
							instancePath: instancePath + "/stormSlowRemainingTicks",
							schemaPath: "#/properties/stormSlowRemainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 180
							},
							message: "must be <= 180"
						};
						if (vErrors === null) vErrors = [err26];
						else vErrors.push(err26);
						errors++;
					}
					if (data16 < 0 || isNaN(data16)) {
						const err27 = {
							instancePath: instancePath + "/stormSlowRemainingTicks",
							schemaPath: "#/properties/stormSlowRemainingTicks/minimum",
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
		} else {
			const err28 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err28];
			else vErrors.push(err28);
			errors++;
		}
		validate182.errors = vErrors;
		return errors === 0;
	}
	function validate222(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate183(data.id, {
					instancePath: instancePath + "/id",
					parentData: data,
					parentDataProperty: "id",
					rootData
				})) {
					vErrors = vErrors === null ? validate183.errors : vErrors.concat(validate183.errors);
					errors = vErrors.length;
				}
			}
			if (data.seat !== void 0) {
				if (!validate51(data.seat, {
					instancePath: instancePath + "/seat",
					parentData: data,
					parentDataProperty: "seat",
					rootData
				})) {
					vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
					errors = vErrors.length;
				}
			}
			if (data.alive !== void 0) {
				if (typeof data.alive !== "boolean") {
					const err6 = {
						instancePath: instancePath + "/alive",
						schemaPath: "#/properties/alive/type",
						keyword: "type",
						params: { type: "boolean" },
						message: "must be boolean"
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
		validate222.errors = vErrors;
		return errors === 0;
	}
	function validate181(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate182(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate182.errors : vErrors.concat(validate182.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate222(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate222.errors : vErrors.concat(validate222.errors);
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
		validate181.errors = vErrors;
		return errors === 0;
	}
	var schema95 = {
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
	function validate233(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			if (data.paused === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
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
				if ("timed" !== data.type) {
					const err4 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
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
						schemaPath: "#/properties/remainingTicks/type",
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
							schemaPath: "#/properties/remainingTicks/maximum",
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
							schemaPath: "#/properties/remainingTicks/minimum",
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
						schemaPath: "#/properties/paused/type",
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err9];
			else vErrors.push(err9);
			errors++;
		}
		validate233.errors = vErrors;
		return errors === 0;
	}
	function validate235(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.type !== void 0) {
				if ("remote" !== data.type) {
					const err2 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "remote" },
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate235.errors = vErrors;
		return errors === 0;
	}
	function validate237(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			for (const key0 in data) if (!(key0 === "type")) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.type !== void 0) {
				if ("contact" !== data.type) {
					const err2 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/properties/type/const",
						keyword: "const",
						params: { allowedValue: "contact" },
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
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate237.errors = vErrors;
		return errors === 0;
	}
	function validate232(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate233(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate233.errors : vErrors.concat(validate233.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate235(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate235.errors : vErrors.concat(validate235.errors);
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
			if (!validate237(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate237.errors : vErrors.concat(validate237.errors);
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
		validate232.errors = vErrors;
		return errors === 0;
	}
	function validate227(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate183(data.id, {
					instancePath: instancePath + "/id",
					parentData: data,
					parentDataProperty: "id",
					rootData
				})) {
					vErrors = vErrors === null ? validate183.errors : vErrors.concat(validate183.errors);
					errors = vErrors.length;
				}
			}
			if (data.cell !== void 0) {
				if (!validate169(data.cell, {
					instancePath: instancePath + "/cell",
					parentData: data,
					parentDataProperty: "cell",
					rootData
				})) {
					vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
					errors = vErrors.length;
				}
			}
			if (data.owner !== void 0) {
				if (!validate183(data.owner, {
					instancePath: instancePath + "/owner",
					parentData: data,
					parentDataProperty: "owner",
					rootData
				})) {
					vErrors = vErrors === null ? validate183.errors : vErrors.concat(validate183.errors);
					errors = vErrors.length;
				}
			}
			if (data.range !== void 0) {
				let data3 = data.range;
				if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3) && isFinite(data3))) {
					const err8 = {
						instancePath: instancePath + "/range",
						schemaPath: "#/properties/range/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
				if (typeof data3 == "number" && isFinite(data3)) {
					if (data3 > 8 || isNaN(data3)) {
						const err9 = {
							instancePath: instancePath + "/range",
							schemaPath: "#/properties/range/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 8
							},
							message: "must be <= 8"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
					if (data3 < 1 || isNaN(data3)) {
						const err10 = {
							instancePath: instancePath + "/range",
							schemaPath: "#/properties/range/minimum",
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
			if (data.placedAt !== void 0) {
				if (!validate41(data.placedAt, {
					instancePath: instancePath + "/placedAt",
					parentData: data,
					parentDataProperty: "placedAt",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
			}
			if (data.trigger !== void 0) {
				if (!validate232(data.trigger, {
					instancePath: instancePath + "/trigger",
					parentData: data,
					parentDataProperty: "trigger",
					rootData
				})) {
					vErrors = vErrors === null ? validate232.errors : vErrors.concat(validate232.errors);
					errors = vErrors.length;
				}
			}
			if (data.blast !== void 0) {
				let data6 = data.blast;
				if (!(data6 === "standard" || data6 === "piercing" || data6 === "power")) {
					const err11 = {
						instancePath: instancePath + "/blast",
						schemaPath: "#/properties/blast/enum",
						keyword: "enum",
						params: { allowedValues: schema95.properties.blast.enum },
						message: "must be equal to one of the allowed values"
					};
					if (vErrors === null) vErrors = [err11];
					else vErrors.push(err11);
					errors++;
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
		validate227.errors = vErrors;
		return errors === 0;
	}
	function validate241(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate169(data.cell, {
					instancePath: instancePath + "/cell",
					parentData: data,
					parentDataProperty: "cell",
					rootData
				})) {
					vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
					errors = vErrors.length;
				}
			}
			if (data.expiresAt !== void 0) {
				if (!validate41(data.expiresAt, {
					instancePath: instancePath + "/expiresAt",
					parentData: data,
					parentDataProperty: "expiresAt",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err3 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate241.errors = vErrors;
		return errors === 0;
	}
	var schema102 = { "enum": [
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
	function validate247(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		let valid0 = false;
		for (const v0 of schema102.enum) if (func0(data, v0)) {
			valid0 = true;
			break;
		}
		if (!valid0) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema102.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate247.errors = vErrors;
		return errors === 0;
	}
	function validate245(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate169(data.cell, {
					instancePath: instancePath + "/cell",
					parentData: data,
					parentDataProperty: "cell",
					rootData
				})) {
					vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
					errors = vErrors.length;
				}
			}
			if (data.kind !== void 0) {
				if (!validate247(data.kind, {
					instancePath: instancePath + "/kind",
					parentData: data,
					parentDataProperty: "kind",
					rootData
				})) {
					vErrors = vErrors === null ? validate247.errors : vErrors.concat(validate247.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err3 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate245.errors = vErrors;
		return errors === 0;
	}
	var schema104 = { "enum": ["unlit", "burning"] };
	function validate252(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!(data === "unlit" || data === "burning")) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema104.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate252.errors = vErrors;
		return errors === 0;
	}
	function validate250(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate169(data.cell, {
					instancePath: instancePath + "/cell",
					parentData: data,
					parentDataProperty: "cell",
					rootData
				})) {
					vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
					errors = vErrors.length;
				}
			}
			if (data.state !== void 0) {
				if (!validate252(data.state, {
					instancePath: instancePath + "/state",
					parentData: data,
					parentDataProperty: "state",
					rootData
				})) {
					vErrors = vErrors === null ? validate252.errors : vErrors.concat(validate252.errors);
					errors = vErrors.length;
				}
			}
		} else {
			const err3 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err3];
			else vErrors.push(err3);
			errors++;
		}
		validate250.errors = vErrors;
		return errors === 0;
	}
	function validate255(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate186(data.position, {
					instancePath: instancePath + "/position",
					parentData: data,
					parentDataProperty: "position",
					rootData
				})) {
					vErrors = vErrors === null ? validate186.errors : vErrors.concat(validate186.errors);
					errors = vErrors.length;
				}
			}
			if (data.direction !== void 0) {
				if (!validate188(data.direction, {
					instancePath: instancePath + "/direction",
					parentData: data,
					parentDataProperty: "direction",
					rootData
				})) {
					vErrors = vErrors === null ? validate188.errors : vErrors.concat(validate188.errors);
					errors = vErrors.length;
				}
			}
			if (data.legRemainingTicks !== void 0) {
				let data2 = data.legRemainingTicks;
				if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
					const err4 = {
						instancePath: instancePath + "/legRemainingTicks",
						schemaPath: "#/properties/legRemainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data2 == "number" && isFinite(data2)) {
					if (data2 > 128 || isNaN(data2)) {
						const err5 = {
							instancePath: instancePath + "/legRemainingTicks",
							schemaPath: "#/properties/legRemainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 128
							},
							message: "must be <= 128"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data2 < 0 || isNaN(data2)) {
						const err6 = {
							instancePath: instancePath + "/legRemainingTicks",
							schemaPath: "#/properties/legRemainingTicks/minimum",
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
		validate255.errors = vErrors;
		return errors === 0;
	}
	function validate259(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			if (data.spawnRemainingTicks === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "spawnRemainingTicks" },
					message: "must have required property 'spawnRemainingTicks'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.livingMobs === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "livingMobs" },
					message: "must have required property 'livingMobs'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "id" || key0 === "cell" || key0 === "spawnRemainingTicks" || key0 === "livingMobs")) {
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
			if (data.id !== void 0) {
				let data0 = data.id;
				if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0) && isFinite(data0))) {
					const err5 = {
						instancePath: instancePath + "/id",
						schemaPath: "#/properties/id/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
				if (typeof data0 == "number" && isFinite(data0)) {
					if (data0 > 1 || isNaN(data0)) {
						const err6 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/properties/id/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 1
							},
							message: "must be <= 1"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					if (data0 < 0 || isNaN(data0)) {
						const err7 = {
							instancePath: instancePath + "/id",
							schemaPath: "#/properties/id/minimum",
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
			if (data.cell !== void 0) {
				if (!validate169(data.cell, {
					instancePath: instancePath + "/cell",
					parentData: data,
					parentDataProperty: "cell",
					rootData
				})) {
					vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
					errors = vErrors.length;
				}
			}
			if (data.spawnRemainingTicks !== void 0) {
				let data2 = data.spawnRemainingTicks;
				const _errs6 = errors;
				let valid1 = false;
				let passing0 = null;
				const _errs7 = errors;
				if (!(typeof data2 == "number" && !(data2 % 1) && !isNaN(data2) && isFinite(data2))) {
					const err8 = {
						instancePath: instancePath + "/spawnRemainingTicks",
						schemaPath: "#/properties/spawnRemainingTicks/oneOf/0/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err8];
					else vErrors.push(err8);
					errors++;
				}
				if (typeof data2 == "number" && isFinite(data2)) {
					if (data2 > 3600 || isNaN(data2)) {
						const err9 = {
							instancePath: instancePath + "/spawnRemainingTicks",
							schemaPath: "#/properties/spawnRemainingTicks/oneOf/0/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 3600
							},
							message: "must be <= 3600"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
					if (data2 < 0 || isNaN(data2)) {
						const err10 = {
							instancePath: instancePath + "/spawnRemainingTicks",
							schemaPath: "#/properties/spawnRemainingTicks/oneOf/0/minimum",
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
				var _valid0 = _errs7 === errors;
				if (_valid0) {
					valid1 = true;
					passing0 = 0;
				}
				const _errs9 = errors;
				if (data2 !== null) {
					const err11 = {
						instancePath: instancePath + "/spawnRemainingTicks",
						schemaPath: "#/properties/spawnRemainingTicks/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err11];
					else vErrors.push(err11);
					errors++;
				}
				var _valid0 = _errs9 === errors;
				if (_valid0 && valid1) {
					valid1 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid1 = true;
					passing0 = 1;
				}
				if (!valid1) {
					const err12 = {
						instancePath: instancePath + "/spawnRemainingTicks",
						schemaPath: "#/properties/spawnRemainingTicks/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				} else {
					errors = _errs6;
					if (vErrors !== null) {
						if (_errs6) vErrors.length = _errs6;
						else vErrors = null;
					}
				}
			}
			if (data.livingMobs !== void 0) {
				let data3 = data.livingMobs;
				if (!(typeof data3 == "number" && !(data3 % 1) && !isNaN(data3) && isFinite(data3))) {
					const err13 = {
						instancePath: instancePath + "/livingMobs",
						schemaPath: "#/properties/livingMobs/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err13];
					else vErrors.push(err13);
					errors++;
				}
				if (typeof data3 == "number" && isFinite(data3)) {
					if (data3 > 1 || isNaN(data3)) {
						const err14 = {
							instancePath: instancePath + "/livingMobs",
							schemaPath: "#/properties/livingMobs/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 1
							},
							message: "must be <= 1"
						};
						if (vErrors === null) vErrors = [err14];
						else vErrors.push(err14);
						errors++;
					}
					if (data3 < 0 || isNaN(data3)) {
						const err15 = {
							instancePath: instancePath + "/livingMobs",
							schemaPath: "#/properties/livingMobs/minimum",
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
		validate259.errors = vErrors;
		return errors === 0;
	}
	function validate262(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
			if (data.lairId === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "lairId" },
					message: "must have required property 'lairId'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.position === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "position" },
					message: "must have required property 'position'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			if (data.heading === void 0) {
				const err3 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "heading" },
					message: "must have required property 'heading'"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.frozen === void 0) {
				const err4 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: "frozen" },
					message: "must have required property 'frozen'"
				};
				if (vErrors === null) vErrors = [err4];
				else vErrors.push(err4);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "id" || key0 === "lairId" || key0 === "position" || key0 === "heading" || key0 === "frozen")) {
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
			if (data.id !== void 0) {
				if (!validate183(data.id, {
					instancePath: instancePath + "/id",
					parentData: data,
					parentDataProperty: "id",
					rootData
				})) {
					vErrors = vErrors === null ? validate183.errors : vErrors.concat(validate183.errors);
					errors = vErrors.length;
				}
			}
			if (data.lairId !== void 0) {
				let data1 = data.lairId;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err6 = {
						instancePath: instancePath + "/lairId",
						schemaPath: "#/properties/lairId/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 1 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/lairId",
							schemaPath: "#/properties/lairId/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 1
							},
							message: "must be <= 1"
						};
						if (vErrors === null) vErrors = [err7];
						else vErrors.push(err7);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err8 = {
							instancePath: instancePath + "/lairId",
							schemaPath: "#/properties/lairId/minimum",
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
			if (data.position !== void 0) {
				if (!validate186(data.position, {
					instancePath: instancePath + "/position",
					parentData: data,
					parentDataProperty: "position",
					rootData
				})) {
					vErrors = vErrors === null ? validate186.errors : vErrors.concat(validate186.errors);
					errors = vErrors.length;
				}
			}
			if (data.heading !== void 0) {
				if (!validate188(data.heading, {
					instancePath: instancePath + "/heading",
					parentData: data,
					parentDataProperty: "heading",
					rootData
				})) {
					vErrors = vErrors === null ? validate188.errors : vErrors.concat(validate188.errors);
					errors = vErrors.length;
				}
			}
			if (data.frozen !== void 0) {
				if (typeof data.frozen !== "boolean") {
					const err9 = {
						instancePath: instancePath + "/frozen",
						schemaPath: "#/properties/frozen/type",
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
		validate262.errors = vErrors;
		return errors === 0;
	}
	function validate267(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.cell === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/oneOf/0/required",
					keyword: "required",
					params: { missingProperty: "cell" },
					message: "must have required property 'cell'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			if (data.phase === void 0) {
				const err1 = {
					instancePath,
					schemaPath: "#/oneOf/0/required",
					keyword: "required",
					params: { missingProperty: "phase" },
					message: "must have required property 'phase'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.remainingTicks === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/oneOf/0/required",
					keyword: "required",
					params: { missingProperty: "remainingTicks" },
					message: "must have required property 'remainingTicks'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "cell" || key0 === "phase" || key0 === "remainingTicks")) {
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
			if (data.cell !== void 0) {
				if (!validate169(data.cell, {
					instancePath: instancePath + "/cell",
					parentData: data,
					parentDataProperty: "cell",
					rootData
				})) {
					vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
					errors = vErrors.length;
				}
			}
			if (data.phase !== void 0) {
				if ("idle" !== data.phase) {
					const err4 = {
						instancePath: instancePath + "/phase",
						schemaPath: "#/oneOf/0/properties/phase/const",
						keyword: "const",
						params: { allowedValue: "idle" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.remainingTicks !== void 0) {
				if (0 !== data.remainingTicks) {
					const err5 = {
						instancePath: instancePath + "/remainingTicks",
						schemaPath: "#/oneOf/0/properties/remainingTicks/const",
						keyword: "const",
						params: { allowedValue: 0 },
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
		}
		const _errs7 = errors;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			if (data.cell === void 0) {
				const err7 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "cell" },
					message: "must have required property 'cell'"
				};
				if (vErrors === null) vErrors = [err7];
				else vErrors.push(err7);
				errors++;
			}
			if (data.phase === void 0) {
				const err8 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "phase" },
					message: "must have required property 'phase'"
				};
				if (vErrors === null) vErrors = [err8];
				else vErrors.push(err8);
				errors++;
			}
			if (data.remainingTicks === void 0) {
				const err9 = {
					instancePath,
					schemaPath: "#/oneOf/1/required",
					keyword: "required",
					params: { missingProperty: "remainingTicks" },
					message: "must have required property 'remainingTicks'"
				};
				if (vErrors === null) vErrors = [err9];
				else vErrors.push(err9);
				errors++;
			}
			for (const key1 in data) if (!(key1 === "cell" || key1 === "phase" || key1 === "remainingTicks")) {
				const err10 = {
					instancePath,
					schemaPath: "#/oneOf/1/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key1 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err10];
				else vErrors.push(err10);
				errors++;
			}
			if (data.cell !== void 0) {
				if (!validate169(data.cell, {
					instancePath: instancePath + "/cell",
					parentData: data,
					parentDataProperty: "cell",
					rootData
				})) {
					vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
					errors = vErrors.length;
				}
			}
			if (data.phase !== void 0) {
				if ("warning" !== data.phase) {
					const err11 = {
						instancePath: instancePath + "/phase",
						schemaPath: "#/oneOf/1/properties/phase/const",
						keyword: "const",
						params: { allowedValue: "warning" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err11];
					else vErrors.push(err11);
					errors++;
				}
			}
			if (data.remainingTicks !== void 0) {
				let data5 = data.remainingTicks;
				if (!(typeof data5 == "number" && !(data5 % 1) && !isNaN(data5) && isFinite(data5))) {
					const err12 = {
						instancePath: instancePath + "/remainingTicks",
						schemaPath: "#/oneOf/1/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				}
				if (typeof data5 == "number" && isFinite(data5)) {
					if (data5 > 120 || isNaN(data5)) {
						const err13 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/oneOf/1/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 120
							},
							message: "must be <= 120"
						};
						if (vErrors === null) vErrors = [err13];
						else vErrors.push(err13);
						errors++;
					}
					if (data5 < 1 || isNaN(data5)) {
						const err14 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/oneOf/1/properties/remainingTicks/minimum",
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
				schemaPath: "#/oneOf/1/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err15];
			else vErrors.push(err15);
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
			const _errs14 = errors;
			if (data && typeof data == "object" && !Array.isArray(data)) {
				if (data.cell === void 0) {
					const err16 = {
						instancePath,
						schemaPath: "#/oneOf/2/required",
						keyword: "required",
						params: { missingProperty: "cell" },
						message: "must have required property 'cell'"
					};
					if (vErrors === null) vErrors = [err16];
					else vErrors.push(err16);
					errors++;
				}
				if (data.phase === void 0) {
					const err17 = {
						instancePath,
						schemaPath: "#/oneOf/2/required",
						keyword: "required",
						params: { missingProperty: "phase" },
						message: "must have required property 'phase'"
					};
					if (vErrors === null) vErrors = [err17];
					else vErrors.push(err17);
					errors++;
				}
				if (data.remainingTicks === void 0) {
					const err18 = {
						instancePath,
						schemaPath: "#/oneOf/2/required",
						keyword: "required",
						params: { missingProperty: "remainingTicks" },
						message: "must have required property 'remainingTicks'"
					};
					if (vErrors === null) vErrors = [err18];
					else vErrors.push(err18);
					errors++;
				}
				for (const key2 in data) if (!(key2 === "cell" || key2 === "phase" || key2 === "remainingTicks")) {
					const err19 = {
						instancePath,
						schemaPath: "#/oneOf/2/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key2 },
						message: "must NOT have additional properties"
					};
					if (vErrors === null) vErrors = [err19];
					else vErrors.push(err19);
					errors++;
				}
				if (data.cell !== void 0) {
					if (!validate169(data.cell, {
						instancePath: instancePath + "/cell",
						parentData: data,
						parentDataProperty: "cell",
						rootData
					})) {
						vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
						errors = vErrors.length;
					}
				}
				if (data.phase !== void 0) {
					if ("raised" !== data.phase) {
						const err20 = {
							instancePath: instancePath + "/phase",
							schemaPath: "#/oneOf/2/properties/phase/const",
							keyword: "const",
							params: { allowedValue: "raised" },
							message: "must be equal to constant"
						};
						if (vErrors === null) vErrors = [err20];
						else vErrors.push(err20);
						errors++;
					}
				}
				if (data.remainingTicks !== void 0) {
					let data8 = data.remainingTicks;
					if (!(typeof data8 == "number" && !(data8 % 1) && !isNaN(data8) && isFinite(data8))) {
						const err21 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/oneOf/2/properties/remainingTicks/type",
							keyword: "type",
							params: { type: "integer" },
							message: "must be integer"
						};
						if (vErrors === null) vErrors = [err21];
						else vErrors.push(err21);
						errors++;
					}
					if (typeof data8 == "number" && isFinite(data8)) {
						if (data8 > 600 || isNaN(data8)) {
							const err22 = {
								instancePath: instancePath + "/remainingTicks",
								schemaPath: "#/oneOf/2/properties/remainingTicks/maximum",
								keyword: "maximum",
								params: {
									comparison: "<=",
									limit: 600
								},
								message: "must be <= 600"
							};
							if (vErrors === null) vErrors = [err22];
							else vErrors.push(err22);
							errors++;
						}
						if (data8 < 1 || isNaN(data8)) {
							const err23 = {
								instancePath: instancePath + "/remainingTicks",
								schemaPath: "#/oneOf/2/properties/remainingTicks/minimum",
								keyword: "minimum",
								params: {
									comparison: ">=",
									limit: 1
								},
								message: "must be >= 1"
							};
							if (vErrors === null) vErrors = [err23];
							else vErrors.push(err23);
							errors++;
						}
					}
				}
			} else {
				const err24 = {
					instancePath,
					schemaPath: "#/oneOf/2/type",
					keyword: "type",
					params: { type: "object" },
					message: "must be object"
				};
				if (vErrors === null) vErrors = [err24];
				else vErrors.push(err24);
				errors++;
			}
			var _valid0 = _errs14 === errors;
			if (_valid0 && valid0) {
				valid0 = false;
				passing0 = [passing0, 2];
			} else if (_valid0) {
				valid0 = true;
				passing0 = 2;
			}
		}
		if (!valid0) {
			const err25 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err25];
			else vErrors.push(err25);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate267.errors = vErrors;
		return errors === 0;
	}
	function validate163(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (data && typeof data == "object" && !Array.isArray(data)) {
			for (const prop0 of schema67.required) if (data[prop0] === void 0) {
				const err0 = {
					instancePath,
					schemaPath: "#/required",
					keyword: "required",
					params: { missingProperty: prop0 },
					message: "must have required property '" + prop0 + "'"
				};
				if (vErrors === null) vErrors = [err0];
				else vErrors.push(err0);
				errors++;
			}
			for (const key0 in data) if (!func2.call(schema67.properties, key0)) {
				const err1 = {
					instancePath,
					schemaPath: "#/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.runningUpdates !== void 0) {
				let data0 = data.runningUpdates;
				if (!(typeof data0 == "number" && !(data0 % 1) && !isNaN(data0) && isFinite(data0))) {
					const err2 = {
						instancePath: instancePath + "/runningUpdates",
						schemaPath: "#/properties/runningUpdates/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err2];
					else vErrors.push(err2);
					errors++;
				}
				if (typeof data0 == "number" && isFinite(data0)) {
					if (data0 > 10800 || isNaN(data0)) {
						const err3 = {
							instancePath: instancePath + "/runningUpdates",
							schemaPath: "#/properties/runningUpdates/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 10800
							},
							message: "must be <= 10800"
						};
						if (vErrors === null) vErrors = [err3];
						else vErrors.push(err3);
						errors++;
					}
					if (data0 < 0 || isNaN(data0)) {
						const err4 = {
							instancePath: instancePath + "/runningUpdates",
							schemaPath: "#/properties/runningUpdates/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
						};
						if (vErrors === null) vErrors = [err4];
						else vErrors.push(err4);
						errors++;
					}
				}
			}
			if (data.remainingTicks !== void 0) {
				let data1 = data.remainingTicks;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err5 = {
						instancePath: instancePath + "/remainingTicks",
						schemaPath: "#/properties/remainingTicks/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 10800 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 10800
							},
							message: "must be <= 10800"
						};
						if (vErrors === null) vErrors = [err6];
						else vErrors.push(err6);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err7 = {
							instancePath: instancePath + "/remainingTicks",
							schemaPath: "#/properties/remainingTicks/minimum",
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
			if (data.physicalTick !== void 0) {
				if (!validate41(data.physicalTick, {
					instancePath: instancePath + "/physicalTick",
					parentData: data,
					parentDataProperty: "physicalTick",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
			}
			if (data.arena !== void 0) {
				if (!validate165(data.arena, {
					instancePath: instancePath + "/arena",
					parentData: data,
					parentDataProperty: "arena",
					rootData
				})) {
					vErrors = vErrors === null ? validate165.errors : vErrors.concat(validate165.errors);
					errors = vErrors.length;
				}
			}
			if (data.actors !== void 0) {
				let data4 = data.actors;
				if (Array.isArray(data4)) {
					if (data4.length > 4) {
						const err8 = {
							instancePath: instancePath + "/actors",
							schemaPath: "#/properties/actors/maxItems",
							keyword: "maxItems",
							params: { limit: 4 },
							message: "must NOT have more than 4 items"
						};
						if (vErrors === null) vErrors = [err8];
						else vErrors.push(err8);
						errors++;
					}
					if (data4.length < 2) {
						const err9 = {
							instancePath: instancePath + "/actors",
							schemaPath: "#/properties/actors/minItems",
							keyword: "minItems",
							params: { limit: 2 },
							message: "must NOT have fewer than 2 items"
						};
						if (vErrors === null) vErrors = [err9];
						else vErrors.push(err9);
						errors++;
					}
					const len0 = data4.length;
					for (let i0 = 0; i0 < len0; i0++) if (!validate181(data4[i0], {
						instancePath: instancePath + "/actors/" + i0,
						parentData: data4,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate181.errors : vErrors.concat(validate181.errors);
						errors = vErrors.length;
					}
				} else {
					const err10 = {
						instancePath: instancePath + "/actors",
						schemaPath: "#/properties/actors/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				}
			}
			if (data.bombs !== void 0) {
				let data6 = data.bombs;
				if (Array.isArray(data6)) {
					if (data6.length > 195) {
						const err11 = {
							instancePath: instancePath + "/bombs",
							schemaPath: "#/properties/bombs/maxItems",
							keyword: "maxItems",
							params: { limit: 195 },
							message: "must NOT have more than 195 items"
						};
						if (vErrors === null) vErrors = [err11];
						else vErrors.push(err11);
						errors++;
					}
					if (data6.length < 0) {
						const err12 = {
							instancePath: instancePath + "/bombs",
							schemaPath: "#/properties/bombs/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err12];
						else vErrors.push(err12);
						errors++;
					}
					const len1 = data6.length;
					for (let i1 = 0; i1 < len1; i1++) if (!validate227(data6[i1], {
						instancePath: instancePath + "/bombs/" + i1,
						parentData: data6,
						parentDataProperty: i1,
						rootData
					})) {
						vErrors = vErrors === null ? validate227.errors : vErrors.concat(validate227.errors);
						errors = vErrors.length;
					}
				} else {
					const err13 = {
						instancePath: instancePath + "/bombs",
						schemaPath: "#/properties/bombs/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err13];
					else vErrors.push(err13);
					errors++;
				}
			}
			if (data.flames !== void 0) {
				let data8 = data.flames;
				if (Array.isArray(data8)) {
					if (data8.length > 195) {
						const err14 = {
							instancePath: instancePath + "/flames",
							schemaPath: "#/properties/flames/maxItems",
							keyword: "maxItems",
							params: { limit: 195 },
							message: "must NOT have more than 195 items"
						};
						if (vErrors === null) vErrors = [err14];
						else vErrors.push(err14);
						errors++;
					}
					if (data8.length < 0) {
						const err15 = {
							instancePath: instancePath + "/flames",
							schemaPath: "#/properties/flames/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err15];
						else vErrors.push(err15);
						errors++;
					}
					const len2 = data8.length;
					for (let i2 = 0; i2 < len2; i2++) if (!validate241(data8[i2], {
						instancePath: instancePath + "/flames/" + i2,
						parentData: data8,
						parentDataProperty: i2,
						rootData
					})) {
						vErrors = vErrors === null ? validate241.errors : vErrors.concat(validate241.errors);
						errors = vErrors.length;
					}
				} else {
					const err16 = {
						instancePath: instancePath + "/flames",
						schemaPath: "#/properties/flames/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err16];
					else vErrors.push(err16);
					errors++;
				}
			}
			if (data.pickups !== void 0) {
				let data10 = data.pickups;
				if (Array.isArray(data10)) {
					if (data10.length > 195) {
						const err17 = {
							instancePath: instancePath + "/pickups",
							schemaPath: "#/properties/pickups/maxItems",
							keyword: "maxItems",
							params: { limit: 195 },
							message: "must NOT have more than 195 items"
						};
						if (vErrors === null) vErrors = [err17];
						else vErrors.push(err17);
						errors++;
					}
					if (data10.length < 0) {
						const err18 = {
							instancePath: instancePath + "/pickups",
							schemaPath: "#/properties/pickups/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err18];
						else vErrors.push(err18);
						errors++;
					}
					const len3 = data10.length;
					for (let i3 = 0; i3 < len3; i3++) if (!validate245(data10[i3], {
						instancePath: instancePath + "/pickups/" + i3,
						parentData: data10,
						parentDataProperty: i3,
						rootData
					})) {
						vErrors = vErrors === null ? validate245.errors : vErrors.concat(validate245.errors);
						errors = vErrors.length;
					}
					let i4 = data10.length;
					let j0;
					if (i4 > 1) {
						outer0: for (; i4--;) for (j0 = i4; j0--;) if (func0(data10[i4], data10[j0])) {
							const err19 = {
								instancePath: instancePath + "/pickups",
								schemaPath: "#/properties/pickups/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i4,
									j: j0
								},
								message: "must NOT have duplicate items (items ## " + j0 + " and " + i4 + " are identical)"
							};
							if (vErrors === null) vErrors = [err19];
							else vErrors.push(err19);
							errors++;
							break outer0;
						}
					}
				} else {
					const err20 = {
						instancePath: instancePath + "/pickups",
						schemaPath: "#/properties/pickups/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err20];
					else vErrors.push(err20);
					errors++;
				}
			}
			if (data.oilBarrels !== void 0) {
				let data12 = data.oilBarrels;
				if (Array.isArray(data12)) {
					if (data12.length > 8) {
						const err21 = {
							instancePath: instancePath + "/oilBarrels",
							schemaPath: "#/properties/oilBarrels/maxItems",
							keyword: "maxItems",
							params: { limit: 8 },
							message: "must NOT have more than 8 items"
						};
						if (vErrors === null) vErrors = [err21];
						else vErrors.push(err21);
						errors++;
					}
					if (data12.length < 0) {
						const err22 = {
							instancePath: instancePath + "/oilBarrels",
							schemaPath: "#/properties/oilBarrels/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err22];
						else vErrors.push(err22);
						errors++;
					}
					const len4 = data12.length;
					for (let i5 = 0; i5 < len4; i5++) if (!validate250(data12[i5], {
						instancePath: instancePath + "/oilBarrels/" + i5,
						parentData: data12,
						parentDataProperty: i5,
						rootData
					})) {
						vErrors = vErrors === null ? validate250.errors : vErrors.concat(validate250.errors);
						errors = vErrors.length;
					}
					let i6 = data12.length;
					let j1;
					if (i6 > 1) {
						outer1: for (; i6--;) for (j1 = i6; j1--;) if (func0(data12[i6], data12[j1])) {
							const err23 = {
								instancePath: instancePath + "/oilBarrels",
								schemaPath: "#/properties/oilBarrels/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i6,
									j: j1
								},
								message: "must NOT have duplicate items (items ## " + j1 + " and " + i6 + " are identical)"
							};
							if (vErrors === null) vErrors = [err23];
							else vErrors.push(err23);
							errors++;
							break outer1;
						}
					}
				} else {
					const err24 = {
						instancePath: instancePath + "/oilBarrels",
						schemaPath: "#/properties/oilBarrels/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err24];
					else vErrors.push(err24);
					errors++;
				}
			}
			if (data.stormCloud !== void 0) {
				let data14 = data.stormCloud;
				const _errs24 = errors;
				let valid13 = false;
				let passing0 = null;
				const _errs25 = errors;
				if (!validate255(data14, {
					instancePath: instancePath + "/stormCloud",
					parentData: data,
					parentDataProperty: "stormCloud",
					rootData
				})) {
					vErrors = vErrors === null ? validate255.errors : vErrors.concat(validate255.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs25 === errors;
				if (_valid0) {
					valid13 = true;
					passing0 = 0;
				}
				const _errs26 = errors;
				if (data14 !== null) {
					const err25 = {
						instancePath: instancePath + "/stormCloud",
						schemaPath: "#/properties/stormCloud/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err25];
					else vErrors.push(err25);
					errors++;
				}
				var _valid0 = _errs26 === errors;
				if (_valid0 && valid13) {
					valid13 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid13 = true;
					passing0 = 1;
				}
				if (!valid13) {
					const err26 = {
						instancePath: instancePath + "/stormCloud",
						schemaPath: "#/properties/stormCloud/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err26];
					else vErrors.push(err26);
					errors++;
				} else {
					errors = _errs24;
					if (vErrors !== null) {
						if (_errs24) vErrors.length = _errs24;
						else vErrors = null;
					}
				}
			}
			if (data.lairs !== void 0) {
				let data15 = data.lairs;
				if (Array.isArray(data15)) {
					if (data15.length > 2) {
						const err27 = {
							instancePath: instancePath + "/lairs",
							schemaPath: "#/properties/lairs/maxItems",
							keyword: "maxItems",
							params: { limit: 2 },
							message: "must NOT have more than 2 items"
						};
						if (vErrors === null) vErrors = [err27];
						else vErrors.push(err27);
						errors++;
					}
					if (data15.length < 0) {
						const err28 = {
							instancePath: instancePath + "/lairs",
							schemaPath: "#/properties/lairs/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err28];
						else vErrors.push(err28);
						errors++;
					}
					const len5 = data15.length;
					for (let i7 = 0; i7 < len5; i7++) if (!validate259(data15[i7], {
						instancePath: instancePath + "/lairs/" + i7,
						parentData: data15,
						parentDataProperty: i7,
						rootData
					})) {
						vErrors = vErrors === null ? validate259.errors : vErrors.concat(validate259.errors);
						errors = vErrors.length;
					}
				} else {
					const err29 = {
						instancePath: instancePath + "/lairs",
						schemaPath: "#/properties/lairs/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err29];
					else vErrors.push(err29);
					errors++;
				}
			}
			if (data.mobs !== void 0) {
				let data17 = data.mobs;
				if (Array.isArray(data17)) {
					if (data17.length > 2) {
						const err30 = {
							instancePath: instancePath + "/mobs",
							schemaPath: "#/properties/mobs/maxItems",
							keyword: "maxItems",
							params: { limit: 2 },
							message: "must NOT have more than 2 items"
						};
						if (vErrors === null) vErrors = [err30];
						else vErrors.push(err30);
						errors++;
					}
					if (data17.length < 0) {
						const err31 = {
							instancePath: instancePath + "/mobs",
							schemaPath: "#/properties/mobs/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err31];
						else vErrors.push(err31);
						errors++;
					}
					const len6 = data17.length;
					for (let i8 = 0; i8 < len6; i8++) if (!validate262(data17[i8], {
						instancePath: instancePath + "/mobs/" + i8,
						parentData: data17,
						parentDataProperty: i8,
						rootData
					})) {
						vErrors = vErrors === null ? validate262.errors : vErrors.concat(validate262.errors);
						errors = vErrors.length;
					}
				} else {
					const err32 = {
						instancePath: instancePath + "/mobs",
						schemaPath: "#/properties/mobs/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err32];
					else vErrors.push(err32);
					errors++;
				}
			}
			if (data.spikePits !== void 0) {
				let data19 = data.spikePits;
				if (Array.isArray(data19)) {
					if (data19.length > 4) {
						const err33 = {
							instancePath: instancePath + "/spikePits",
							schemaPath: "#/properties/spikePits/maxItems",
							keyword: "maxItems",
							params: { limit: 4 },
							message: "must NOT have more than 4 items"
						};
						if (vErrors === null) vErrors = [err33];
						else vErrors.push(err33);
						errors++;
					}
					if (data19.length < 0) {
						const err34 = {
							instancePath: instancePath + "/spikePits",
							schemaPath: "#/properties/spikePits/minItems",
							keyword: "minItems",
							params: { limit: 0 },
							message: "must NOT have fewer than 0 items"
						};
						if (vErrors === null) vErrors = [err34];
						else vErrors.push(err34);
						errors++;
					}
					const len7 = data19.length;
					for (let i9 = 0; i9 < len7; i9++) if (!validate267(data19[i9], {
						instancePath: instancePath + "/spikePits/" + i9,
						parentData: data19,
						parentDataProperty: i9,
						rootData
					})) {
						vErrors = vErrors === null ? validate267.errors : vErrors.concat(validate267.errors);
						errors = vErrors.length;
					}
					let i10 = data19.length;
					let j2;
					if (i10 > 1) {
						outer2: for (; i10--;) for (j2 = i10; j2--;) if (func0(data19[i10], data19[j2])) {
							const err35 = {
								instancePath: instancePath + "/spikePits",
								schemaPath: "#/properties/spikePits/uniqueItems",
								keyword: "uniqueItems",
								params: {
									i: i10,
									j: j2
								},
								message: "must NOT have duplicate items (items ## " + j2 + " and " + i10 + " are identical)"
							};
							if (vErrors === null) vErrors = [err35];
							else vErrors.push(err35);
							errors++;
							break outer2;
						}
					}
				} else {
					const err36 = {
						instancePath: instancePath + "/spikePits",
						schemaPath: "#/properties/spikePits/type",
						keyword: "type",
						params: { type: "array" },
						message: "must be array"
					};
					if (vErrors === null) vErrors = [err36];
					else vErrors.push(err36);
					errors++;
				}
			}
		} else {
			const err37 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err37];
			else vErrors.push(err37);
			errors++;
		}
		validate163.errors = vErrors;
		return errors === 0;
	}
	function validate31(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate32(data.sequence, {
					instancePath: instancePath + "/sequence",
					parentData: data,
					parentDataProperty: "sequence",
					rootData
				})) {
					vErrors = vErrors === null ? validate32.errors : vErrors.concat(validate32.errors);
					errors = vErrors.length;
				}
			}
			if (data.room !== void 0) {
				if (!validate34(data.room, {
					instancePath: instancePath + "/room",
					parentData: data,
					parentDataProperty: "room",
					rootData
				})) {
					vErrors = vErrors === null ? validate34.errors : vErrors.concat(validate34.errors);
					errors = vErrors.length;
				}
			}
			if (data.youSeat !== void 0) {
				if (!validate51(data.youSeat, {
					instancePath: instancePath + "/youSeat",
					parentData: data,
					parentDataProperty: "youSeat",
					rootData
				})) {
					vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
					errors = vErrors.length;
				}
			}
			if (data.resumeToken !== void 0) {
				if (!validate161(data.resumeToken, {
					instancePath: instancePath + "/resumeToken",
					parentData: data,
					parentDataProperty: "resumeToken",
					rootData
				})) {
					vErrors = vErrors === null ? validate161.errors : vErrors.concat(validate161.errors);
					errors = vErrors.length;
				}
			}
			if (data.state !== void 0) {
				let data5 = data.state;
				const _errs8 = errors;
				let valid1 = false;
				let passing0 = null;
				const _errs9 = errors;
				if (!validate163(data5, {
					instancePath: instancePath + "/state",
					parentData: data,
					parentDataProperty: "state",
					rootData
				})) {
					vErrors = vErrors === null ? validate163.errors : vErrors.concat(validate163.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs9 === errors;
				if (_valid0) {
					valid1 = true;
					passing0 = 0;
				}
				const _errs10 = errors;
				if (data5 !== null) {
					const err9 = {
						instancePath: instancePath + "/state",
						schemaPath: "#/properties/state/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err9];
					else vErrors.push(err9);
					errors++;
				}
				var _valid0 = _errs10 === errors;
				if (_valid0 && valid1) {
					valid1 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid1 = true;
					passing0 = 1;
				}
				if (!valid1) {
					const err10 = {
						instancePath: instancePath + "/state",
						schemaPath: "#/properties/state/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err10];
					else vErrors.push(err10);
					errors++;
				} else {
					errors = _errs8;
					if (vErrors !== null) {
						if (_errs8) vErrors.length = _errs8;
						else vErrors = null;
					}
				}
			}
			if (data.lastActionId !== void 0) {
				let data6 = data.lastActionId;
				const _errs13 = errors;
				let valid2 = false;
				let passing1 = null;
				const _errs14 = errors;
				if (!validate32(data6, {
					instancePath: instancePath + "/lastActionId",
					parentData: data,
					parentDataProperty: "lastActionId",
					rootData
				})) {
					vErrors = vErrors === null ? validate32.errors : vErrors.concat(validate32.errors);
					errors = vErrors.length;
				}
				var _valid1 = _errs14 === errors;
				if (_valid1) {
					valid2 = true;
					passing1 = 0;
				}
				const _errs15 = errors;
				if (data6 !== null) {
					const err11 = {
						instancePath: instancePath + "/lastActionId",
						schemaPath: "#/properties/lastActionId/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err11];
					else vErrors.push(err11);
					errors++;
				}
				var _valid1 = _errs15 === errors;
				if (_valid1 && valid2) {
					valid2 = false;
					passing1 = [passing1, 1];
				} else if (_valid1) {
					valid2 = true;
					passing1 = 1;
				}
				if (!valid2) {
					const err12 = {
						instancePath: instancePath + "/lastActionId",
						schemaPath: "#/properties/lastActionId/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing1 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				} else {
					errors = _errs13;
					if (vErrors !== null) {
						if (_errs13) vErrors.length = _errs13;
						else vErrors = null;
					}
				}
			}
		} else {
			const err13 = {
				instancePath,
				schemaPath: "#/type",
				keyword: "type",
				params: { type: "object" },
				message: "must be object"
			};
			if (vErrors === null) vErrors = [err13];
			else vErrors.push(err13);
			errors++;
		}
		validate31.errors = vErrors;
		return errors === 0;
	}
	function validate275(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate34(data.room, {
					instancePath: instancePath + "/room",
					parentData: data,
					parentDataProperty: "room",
					rootData
				})) {
					vErrors = vErrors === null ? validate34.errors : vErrors.concat(validate34.errors);
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
		validate275.errors = vErrors;
		return errors === 0;
	}
	function validate284(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate227(data.bomb, {
					instancePath: instancePath + "/bomb",
					parentData: data,
					parentDataProperty: "bomb",
					rootData
				})) {
					vErrors = vErrors === null ? validate227.errors : vErrors.concat(validate227.errors);
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
		validate284.errors = vErrors;
		return errors === 0;
	}
	function validate287(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate227(data.bomb, {
					instancePath: instancePath + "/bomb",
					parentData: data,
					parentDataProperty: "bomb",
					rootData
				})) {
					vErrors = vErrors === null ? validate227.errors : vErrors.concat(validate227.errors);
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
		validate287.errors = vErrors;
		return errors === 0;
	}
	function validate290(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate183(data.id, {
					instancePath: instancePath + "/id",
					parentData: data,
					parentDataProperty: "id",
					rootData
				})) {
					vErrors = vErrors === null ? validate183.errors : vErrors.concat(validate183.errors);
					errors = vErrors.length;
				}
			}
			if (data.seat !== void 0) {
				if (!validate51(data.seat, {
					instancePath: instancePath + "/seat",
					parentData: data,
					parentDataProperty: "seat",
					rootData
				})) {
					vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
					errors = vErrors.length;
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
		validate290.errors = vErrors;
		return errors === 0;
	}
	function validate294(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate61(data.result, {
					instancePath: instancePath + "/result",
					parentData: data,
					parentDataProperty: "result",
					rootData
				})) {
					vErrors = vErrors === null ? validate61.errors : vErrors.concat(validate61.errors);
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
		validate294.errors = vErrors;
		return errors === 0;
	}
	function validate297(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate183(data.id, {
					instancePath: instancePath + "/id",
					parentData: data,
					parentDataProperty: "id",
					rootData
				})) {
					vErrors = vErrors === null ? validate183.errors : vErrors.concat(validate183.errors);
					errors = vErrors.length;
				}
			}
			if (data.seat !== void 0) {
				if (!validate51(data.seat, {
					instancePath: instancePath + "/seat",
					parentData: data,
					parentDataProperty: "seat",
					rootData
				})) {
					vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
					errors = vErrors.length;
				}
			}
			if (data.cell !== void 0) {
				if (!validate169(data.cell, {
					instancePath: instancePath + "/cell",
					parentData: data,
					parentDataProperty: "cell",
					rootData
				})) {
					vErrors = vErrors === null ? validate169.errors : vErrors.concat(validate169.errors);
					errors = vErrors.length;
				}
			}
			if (data.kind !== void 0) {
				if (!validate247(data.kind, {
					instancePath: instancePath + "/kind",
					parentData: data,
					parentDataProperty: "kind",
					rootData
				})) {
					vErrors = vErrors === null ? validate247.errors : vErrors.concat(validate247.errors);
					errors = vErrors.length;
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
		validate297.errors = vErrors;
		return errors === 0;
	}
	var schema118 = {
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
	function validate303(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate183(data.id, {
					instancePath: instancePath + "/id",
					parentData: data,
					parentDataProperty: "id",
					rootData
				})) {
					vErrors = vErrors === null ? validate183.errors : vErrors.concat(validate183.errors);
					errors = vErrors.length;
				}
			}
			if (data.seat !== void 0) {
				if (!validate51(data.seat, {
					instancePath: instancePath + "/seat",
					parentData: data,
					parentDataProperty: "seat",
					rootData
				})) {
					vErrors = vErrors === null ? validate51.errors : vErrors.concat(validate51.errors);
					errors = vErrors.length;
				}
			}
			if (data.source !== void 0) {
				let data3 = data.source;
				if (!(data3 === "heart" || data3 === "shield")) {
					const err6 = {
						instancePath: instancePath + "/source",
						schemaPath: "#/properties/source/enum",
						keyword: "enum",
						params: { allowedValues: schema118.properties.source.enum },
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
		validate303.errors = vErrors;
		return errors === 0;
	}
	function validate283(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate284(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate284.errors : vErrors.concat(validate284.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate287(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate287.errors : vErrors.concat(validate287.errors);
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
			if (!validate290(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate290.errors : vErrors.concat(validate290.errors);
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
				if (!validate294(data, {
					instancePath,
					parentData,
					parentDataProperty,
					rootData
				})) {
					vErrors = vErrors === null ? validate294.errors : vErrors.concat(validate294.errors);
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
					if (!validate297(data, {
						instancePath,
						parentData,
						parentDataProperty,
						rootData
					})) {
						vErrors = vErrors === null ? validate297.errors : vErrors.concat(validate297.errors);
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
						if (!validate303(data, {
							instancePath,
							parentData,
							parentDataProperty,
							rootData
						})) {
							vErrors = vErrors === null ? validate303.errors : vErrors.concat(validate303.errors);
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
		validate283.errors = vErrors;
		return errors === 0;
	}
	function validate281(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate41(data.tick, {
					instancePath: instancePath + "/tick",
					parentData: data,
					parentDataProperty: "tick",
					rootData
				})) {
					vErrors = vErrors === null ? validate41.errors : vErrors.concat(validate41.errors);
					errors = vErrors.length;
				}
			}
			if (data.index !== void 0) {
				let data1 = data.index;
				if (!(typeof data1 == "number" && !(data1 % 1) && !isNaN(data1) && isFinite(data1))) {
					const err4 = {
						instancePath: instancePath + "/index",
						schemaPath: "#/properties/index/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
				if (typeof data1 == "number" && isFinite(data1)) {
					if (data1 > 215 || isNaN(data1)) {
						const err5 = {
							instancePath: instancePath + "/index",
							schemaPath: "#/properties/index/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 215
							},
							message: "must be <= 215"
						};
						if (vErrors === null) vErrors = [err5];
						else vErrors.push(err5);
						errors++;
					}
					if (data1 < 0 || isNaN(data1)) {
						const err6 = {
							instancePath: instancePath + "/index",
							schemaPath: "#/properties/index/minimum",
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
			if (data.kind !== void 0) {
				if (!validate283(data.kind, {
					instancePath: instancePath + "/kind",
					parentData: data,
					parentDataProperty: "kind",
					rootData
				})) {
					vErrors = vErrors === null ? validate283.errors : vErrors.concat(validate283.errors);
					errors = vErrors.length;
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
		validate281.errors = vErrors;
		return errors === 0;
	}
	function validate278(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate34(data.room, {
					instancePath: instancePath + "/room",
					parentData: data,
					parentDataProperty: "room",
					rootData
				})) {
					vErrors = vErrors === null ? validate34.errors : vErrors.concat(validate34.errors);
					errors = vErrors.length;
				}
			}
			if (data.state !== void 0) {
				if (!validate163(data.state, {
					instancePath: instancePath + "/state",
					parentData: data,
					parentDataProperty: "state",
					rootData
				})) {
					vErrors = vErrors === null ? validate163.errors : vErrors.concat(validate163.errors);
					errors = vErrors.length;
				}
			}
			if (data.events !== void 0) {
				let data3 = data.events;
				if (Array.isArray(data3)) {
					if (data3.length > 256) {
						const err6 = {
							instancePath: instancePath + "/events",
							schemaPath: "#/properties/events/maxItems",
							keyword: "maxItems",
							params: { limit: 256 },
							message: "must NOT have more than 256 items"
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
					for (let i0 = 0; i0 < len0; i0++) if (!validate281(data3[i0], {
						instancePath: instancePath + "/events/" + i0,
						parentData: data3,
						parentDataProperty: i0,
						rootData
					})) {
						vErrors = vErrors === null ? validate281.errors : vErrors.concat(validate281.errors);
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
		validate278.errors = vErrors;
		return errors === 0;
	}
	function validate310(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate35(data.roomId, {
					instancePath: instancePath + "/roomId",
					parentData: data,
					parentDataProperty: "roomId",
					rootData
				})) {
					vErrors = vErrors === null ? validate35.errors : vErrors.concat(validate35.errors);
					errors = vErrors.length;
				}
			}
			if (data.sequence !== void 0) {
				if (!validate32(data.sequence, {
					instancePath: instancePath + "/sequence",
					parentData: data,
					parentDataProperty: "sequence",
					rootData
				})) {
					vErrors = vErrors === null ? validate32.errors : vErrors.concat(validate32.errors);
					errors = vErrors.length;
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
		validate310.errors = vErrors;
		return errors === 0;
	}
	function validate314(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate35(data.roomId, {
					instancePath: instancePath + "/roomId",
					parentData: data,
					parentDataProperty: "roomId",
					rootData
				})) {
					vErrors = vErrors === null ? validate35.errors : vErrors.concat(validate35.errors);
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
		validate314.errors = vErrors;
		return errors === 0;
	}
	var schema122 = { "enum": [
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
	function validate318(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		let valid0 = false;
		for (const v0 of schema122.enum) if (func0(data, v0)) {
			valid0 = true;
			break;
		}
		if (!valid0) {
			const err0 = {
				instancePath,
				schemaPath: "#/enum",
				keyword: "enum",
				params: { allowedValues: schema122.enum },
				message: "must be equal to one of the allowed values"
			};
			if (vErrors === null) vErrors = [err0];
			else vErrors.push(err0);
			errors++;
		}
		validate318.errors = vErrors;
		return errors === 0;
	}
	function validate317(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
				if (!validate318(data.code, {
					instancePath: instancePath + "/code",
					parentData: data,
					parentDataProperty: "code",
					rootData
				})) {
					vErrors = vErrors === null ? validate318.errors : vErrors.concat(validate318.errors);
					errors = vErrors.length;
				}
			}
			if (data.sequence !== void 0) {
				let data2 = data.sequence;
				const _errs5 = errors;
				let valid1 = false;
				let passing0 = null;
				const _errs6 = errors;
				if (!validate32(data2, {
					instancePath: instancePath + "/sequence",
					parentData: data,
					parentDataProperty: "sequence",
					rootData
				})) {
					vErrors = vErrors === null ? validate32.errors : vErrors.concat(validate32.errors);
					errors = vErrors.length;
				}
				var _valid0 = _errs6 === errors;
				if (_valid0) {
					valid1 = true;
					passing0 = 0;
				}
				const _errs7 = errors;
				if (data2 !== null) {
					const err5 = {
						instancePath: instancePath + "/sequence",
						schemaPath: "#/properties/sequence/oneOf/1/type",
						keyword: "type",
						params: { type: "null" },
						message: "must be null"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
				var _valid0 = _errs7 === errors;
				if (_valid0 && valid1) {
					valid1 = false;
					passing0 = [passing0, 1];
				} else if (_valid0) {
					valid1 = true;
					passing0 = 1;
				}
				if (!valid1) {
					const err6 = {
						instancePath: instancePath + "/sequence",
						schemaPath: "#/properties/sequence/oneOf",
						keyword: "oneOf",
						params: { passingSchemas: passing0 },
						message: "must match exactly one schema in oneOf"
					};
					if (vErrors === null) vErrors = [err6];
					else vErrors.push(err6);
					errors++;
				} else {
					errors = _errs5;
					if (vErrors !== null) {
						if (_errs5) vErrors.length = _errs5;
						else vErrors = null;
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
		validate317.errors = vErrors;
		return errors === 0;
	}
	function validate323(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		const _errs0 = errors;
		let valid0 = false;
		let passing0 = null;
		const _errs1 = errors;
		if (!validate23(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate23.errors : vErrors.concat(validate23.errors);
			errors = vErrors.length;
		}
		var _valid0 = _errs1 === errors;
		if (_valid0) {
			valid0 = true;
			passing0 = 0;
		}
		const _errs2 = errors;
		if (!validate27(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate27.errors : vErrors.concat(validate27.errors);
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
			if (!validate31(data, {
				instancePath,
				parentData,
				parentDataProperty,
				rootData
			})) {
				vErrors = vErrors === null ? validate31.errors : vErrors.concat(validate31.errors);
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
				if (!validate275(data, {
					instancePath,
					parentData,
					parentDataProperty,
					rootData
				})) {
					vErrors = vErrors === null ? validate275.errors : vErrors.concat(validate275.errors);
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
					if (!validate278(data, {
						instancePath,
						parentData,
						parentDataProperty,
						rootData
					})) {
						vErrors = vErrors === null ? validate278.errors : vErrors.concat(validate278.errors);
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
						if (!validate310(data, {
							instancePath,
							parentData,
							parentDataProperty,
							rootData
						})) {
							vErrors = vErrors === null ? validate310.errors : vErrors.concat(validate310.errors);
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
							if (!validate314(data, {
								instancePath,
								parentData,
								parentDataProperty,
								rootData
							})) {
								vErrors = vErrors === null ? validate314.errors : vErrors.concat(validate314.errors);
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
								if (!validate317(data, {
									instancePath,
									parentData,
									parentDataProperty,
									rootData
								})) {
									vErrors = vErrors === null ? validate317.errors : vErrors.concat(validate317.errors);
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
		validate323.errors = vErrors;
		return errors === 0;
	}
}));
//#endregion
export default require_gameplay_v20();
