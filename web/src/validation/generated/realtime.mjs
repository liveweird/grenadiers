//#region \0rolldown/runtime.js
var __commonJSMin = (cb, mod) => () => (mod || (cb((mod = { exports: {} }).exports, mod, mod.exports), cb = null), mod.exports);
//#endregion
//#region ../../../../../grenadiers-validator-entry/realtime.cjs
var require_realtime = /* @__PURE__ */ __commonJSMin(((exports, module) => {
	module.exports = validate10;
	module.exports.default = validate10;
	function validate11(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
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
					schemaPath: "#/$defs/Welcome/required",
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
					schemaPath: "#/$defs/Welcome/required",
					keyword: "required",
					params: { missingProperty: "protocolVersion" },
					message: "must have required property 'protocolVersion'"
				};
				if (vErrors === null) vErrors = [err1];
				else vErrors.push(err1);
				errors++;
			}
			if (data.maxPlayers === void 0) {
				const err2 = {
					instancePath,
					schemaPath: "#/$defs/Welcome/required",
					keyword: "required",
					params: { missingProperty: "maxPlayers" },
					message: "must have required property 'maxPlayers'"
				};
				if (vErrors === null) vErrors = [err2];
				else vErrors.push(err2);
				errors++;
			}
			for (const key0 in data) if (!(key0 === "type" || key0 === "protocolVersion" || key0 === "maxPlayers")) {
				const err3 = {
					instancePath,
					schemaPath: "#/$defs/Welcome/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key0 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err3];
				else vErrors.push(err3);
				errors++;
			}
			if (data.type !== void 0) {
				if ("welcome" !== data.type) {
					const err4 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/$defs/Welcome/properties/type/const",
						keyword: "const",
						params: { allowedValue: "welcome" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err4];
					else vErrors.push(err4);
					errors++;
				}
			}
			if (data.protocolVersion !== void 0) {
				if (1 !== data.protocolVersion) {
					const err5 = {
						instancePath: instancePath + "/protocolVersion",
						schemaPath: "#/$defs/Welcome/properties/protocolVersion/const",
						keyword: "const",
						params: { allowedValue: 1 },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err5];
					else vErrors.push(err5);
					errors++;
				}
			}
			if (data.maxPlayers !== void 0) {
				if (2 !== data.maxPlayers) {
					const err6 = {
						instancePath: instancePath + "/maxPlayers",
						schemaPath: "#/$defs/Welcome/properties/maxPlayers/const",
						keyword: "const",
						params: { allowedValue: 2 },
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
				schemaPath: "#/$defs/Welcome/type",
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
			if (data.type === void 0) {
				const err8 = {
					instancePath,
					schemaPath: "#/$defs/Pong/required",
					keyword: "required",
					params: { missingProperty: "type" },
					message: "must have required property 'type'"
				};
				if (vErrors === null) vErrors = [err8];
				else vErrors.push(err8);
				errors++;
			}
			if (data.sequence === void 0) {
				const err9 = {
					instancePath,
					schemaPath: "#/$defs/Pong/required",
					keyword: "required",
					params: { missingProperty: "sequence" },
					message: "must have required property 'sequence'"
				};
				if (vErrors === null) vErrors = [err9];
				else vErrors.push(err9);
				errors++;
			}
			for (const key1 in data) if (!(key1 === "type" || key1 === "sequence")) {
				const err10 = {
					instancePath,
					schemaPath: "#/$defs/Pong/additionalProperties",
					keyword: "additionalProperties",
					params: { additionalProperty: key1 },
					message: "must NOT have additional properties"
				};
				if (vErrors === null) vErrors = [err10];
				else vErrors.push(err10);
				errors++;
			}
			if (data.type !== void 0) {
				if ("pong" !== data.type) {
					const err11 = {
						instancePath: instancePath + "/type",
						schemaPath: "#/$defs/Pong/properties/type/const",
						keyword: "const",
						params: { allowedValue: "pong" },
						message: "must be equal to constant"
					};
					if (vErrors === null) vErrors = [err11];
					else vErrors.push(err11);
					errors++;
				}
			}
			if (data.sequence !== void 0) {
				let data4 = data.sequence;
				if (!(typeof data4 == "number" && !(data4 % 1) && !isNaN(data4) && isFinite(data4))) {
					const err12 = {
						instancePath: instancePath + "/sequence",
						schemaPath: "#/$defs/Pong/properties/sequence/type",
						keyword: "type",
						params: { type: "integer" },
						message: "must be integer"
					};
					if (vErrors === null) vErrors = [err12];
					else vErrors.push(err12);
					errors++;
				}
				if (typeof data4 == "number" && isFinite(data4)) {
					if (data4 > 2147483647 || isNaN(data4)) {
						const err13 = {
							instancePath: instancePath + "/sequence",
							schemaPath: "#/$defs/Pong/properties/sequence/maximum",
							keyword: "maximum",
							params: {
								comparison: "<=",
								limit: 2147483647
							},
							message: "must be <= 2147483647"
						};
						if (vErrors === null) vErrors = [err13];
						else vErrors.push(err13);
						errors++;
					}
					if (data4 < 0 || isNaN(data4)) {
						const err14 = {
							instancePath: instancePath + "/sequence",
							schemaPath: "#/$defs/Pong/properties/sequence/minimum",
							keyword: "minimum",
							params: {
								comparison: ">=",
								limit: 0
							},
							message: "must be >= 0"
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
				schemaPath: "#/$defs/Pong/type",
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
				if (data.type === void 0) {
					const err16 = {
						instancePath,
						schemaPath: "#/$defs/Error/required",
						keyword: "required",
						params: { missingProperty: "type" },
						message: "must have required property 'type'"
					};
					if (vErrors === null) vErrors = [err16];
					else vErrors.push(err16);
					errors++;
				}
				if (data.code === void 0) {
					const err17 = {
						instancePath,
						schemaPath: "#/$defs/Error/required",
						keyword: "required",
						params: { missingProperty: "code" },
						message: "must have required property 'code'"
					};
					if (vErrors === null) vErrors = [err17];
					else vErrors.push(err17);
					errors++;
				}
				if (data.message === void 0) {
					const err18 = {
						instancePath,
						schemaPath: "#/$defs/Error/required",
						keyword: "required",
						params: { missingProperty: "message" },
						message: "must have required property 'message'"
					};
					if (vErrors === null) vErrors = [err18];
					else vErrors.push(err18);
					errors++;
				}
				for (const key2 in data) if (!(key2 === "type" || key2 === "code" || key2 === "message")) {
					const err19 = {
						instancePath,
						schemaPath: "#/$defs/Error/additionalProperties",
						keyword: "additionalProperties",
						params: { additionalProperty: key2 },
						message: "must NOT have additional properties"
					};
					if (vErrors === null) vErrors = [err19];
					else vErrors.push(err19);
					errors++;
				}
				if (data.type !== void 0) {
					if ("error" !== data.type) {
						const err20 = {
							instancePath: instancePath + "/type",
							schemaPath: "#/$defs/Error/properties/type/const",
							keyword: "const",
							params: { allowedValue: "error" },
							message: "must be equal to constant"
						};
						if (vErrors === null) vErrors = [err20];
						else vErrors.push(err20);
						errors++;
					}
				}
				if (data.code !== void 0) {
					if ("invalidMessage" !== data.code) {
						const err21 = {
							instancePath: instancePath + "/code",
							schemaPath: "#/$defs/Error/properties/code/const",
							keyword: "const",
							params: { allowedValue: "invalidMessage" },
							message: "must be equal to constant"
						};
						if (vErrors === null) vErrors = [err21];
						else vErrors.push(err21);
						errors++;
					}
				}
				if (data.message !== void 0) {
					if ("Invalid client message" !== data.message) {
						const err22 = {
							instancePath: instancePath + "/message",
							schemaPath: "#/$defs/Error/properties/message/const",
							keyword: "const",
							params: { allowedValue: "Invalid client message" },
							message: "must be equal to constant"
						};
						if (vErrors === null) vErrors = [err22];
						else vErrors.push(err22);
						errors++;
					}
				}
			} else {
				const err23 = {
					instancePath,
					schemaPath: "#/$defs/Error/type",
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
			} else if (_valid0) {
				valid0 = true;
				passing0 = 2;
			}
		}
		if (!valid0) {
			const err24 = {
				instancePath,
				schemaPath: "#/oneOf",
				keyword: "oneOf",
				params: { passingSchemas: passing0 },
				message: "must match exactly one schema in oneOf"
			};
			if (vErrors === null) vErrors = [err24];
			else vErrors.push(err24);
			errors++;
		} else {
			errors = _errs0;
			if (vErrors !== null) {
				if (_errs0) vErrors.length = _errs0;
				else vErrors = null;
			}
		}
		validate11.errors = vErrors;
		return errors === 0;
	}
	function validate10(data, { instancePath = "", parentData, parentDataProperty, rootData = data } = {}) {
		let vErrors = null;
		let errors = 0;
		if (!validate11(data, {
			instancePath,
			parentData,
			parentDataProperty,
			rootData
		})) {
			vErrors = vErrors === null ? validate11.errors : vErrors.concat(validate11.errors);
			errors = vErrors.length;
		}
		validate10.errors = vErrors;
		return errors === 0;
	}
}));
//#endregion
export default require_realtime();
