import { cn } from "$lib/utils.js";
import { resolveCommand } from "package-manager-detector/commands";
import CopyButton from "../copy-button/copy-button.svelte";
import ClipboardIcon from "@lucide/svelte/icons/clipboard";
import TerminalIcon from "@lucide/svelte/icons/terminal";
import * as Tooltip from "$lib/components/ui/tooltip";
import * as Tabs from "$lib/components/ui/tabs";
import { tv } from "tailwind-variants";
const style = tv({
	base: "border-border w-full rounded-lg border",
	variants: { variant: {
		default: "bg-card",
		secondary: "bg-secondary/50 border-transparent"
	} }
});
const __lux_template = "\n\n\n\n<div>\n    <div class=\"flex place-items-center justify-between gap-2 border-b border-border py-1 pr-2\">\n        <div class=\"flex place-items-center gap-2 px-2\">\n            <div class=\"flex size-4 place-items-center justify-center bg-foreground opacity-50\">\n                \n            </div>\n            \n        </div>\n        \n    </div>\n    <div class=\"no-scrollbar overflow-x-auto p-3\">\n		<span class=\"font-mono text-sm leading-none font-light text-nowrap text-muted-foreground\">\n			\n		</span>\n    </div>\n</div>\n\n\n";
const __lux_css = ".no-scrollbar.svelte-1950gtd { -ms-overflow-style: none; scrollbar-width: none; }";
const __lux_css_hash = "1950gtd";
const __lux_css_scope = "svelte-1950gtd";
const __lux_has_dynamic = true;
const __lux_stringify = function(value) {
	return typeof value === "string" ? value : value == null ? "" : value + "";
};
const __lux_escape = function(value) {
	return __lux_stringify(value).replace(/[&<>]/g, function(ch) {
		return ch === "&" ? "&amp;" : ch === "<" ? "&lt;" : "&gt;";
	});
};
const __lux_escape_attr = function(value) {
	return __lux_stringify(value).replace(/[&<>"']/g, function(ch) {
		return ch === "&" ? "&amp;" : ch === "<" ? "&lt;" : ch === ">" ? "&gt;" : ch === "\"" ? "&quot;" : "&#39;";
	});
};
const __lux_is_boolean_attr = function(name) {
	return [
		"allowfullscreen",
		"async",
		"autofocus",
		"autoplay",
		"checked",
		"controls",
		"default",
		"defer",
		"disabled",
		"disablepictureinpicture",
		"disableremoteplayback",
		"formnovalidate",
		"indeterminate",
		"inert",
		"ismap",
		"loop",
		"multiple",
		"muted",
		"nomodule",
		"novalidate",
		"open",
		"playsinline",
		"readonly",
		"required",
		"reversed",
		"seamless",
		"selected",
		"webkitdirectory"
	].includes(__lux_stringify(name).toLowerCase());
};
const __lux_attr = function(name, value, is_boolean) {
	return value == null || (is_boolean || __lux_stringify(name).toLowerCase() === "hidden" && value !== "until-found") && !value ? "" : is_boolean || __lux_stringify(name).toLowerCase() === "hidden" && value !== "until-found" ? " " + __lux_stringify(name) : " " + __lux_stringify(name) + "=\"" + __lux_escape_attr(__lux_stringify(name).toLowerCase() === "translate" && value === true ? "yes" : __lux_stringify(name).toLowerCase() === "translate" && value === false ? "no" : value) + "\"";
};
const __lux_attributes = function(attrs) {
	return Object.entries(attrs ?? {}).map(function(__lux_entry) {
		return typeof __lux_entry[1] === "function" || __lux_stringify(__lux_entry[0]).startsWith("$$") ? "" : __lux_attr(__lux_stringify(__lux_entry[0]), __lux_entry[1], __lux_is_boolean_attr(__lux_stringify(__lux_entry[0])));
	}).join("");
};
export { __lux_template as template, __lux_css as css, __lux_css_hash as cssHash, __lux_css_scope as cssScope, __lux_has_dynamic as hasDynamic };
export default {
	template: __lux_template,
	css: __lux_css,
	cssHash: __lux_css_hash,
	cssScope: __lux_css_scope,
	hasDynamic: __lux_has_dynamic,
	render: function __lux_render(_props = {}) {
		_props.__lux_self == null && (_props.__lux_self = __lux_render);
		let { variant = "default", class: className, command, agents = [
			"npm",
			"pnpm",
			"yarn",
			"bun"
		], args, agent = undefined } = _props;
		const cmd = resolveCommand(agent, command, args);
		const commandText = `${cmd?.command} ${cmd?.args.join(" ")}`;
		return function() {
			let __lux_chunks = [];
			__lux_chunks.push("\n\n");
			__lux_chunks.push("\n\n");
			__lux_chunks.push([
				"<div",
				__lux_attr("class", function({ style }) {
					return cn(style({ variant }), className);
				}(_props), false),
				">",
				function() {
					let __lux_chunks = [];
					__lux_chunks.push("\n    ");
					__lux_chunks.push([
						"<div",
						__lux_attr("class", ["flex place-items-center justify-between gap-2 border-b border-border py-1 pr-2"].join(""), false),
						">",
						function() {
							let __lux_chunks = [];
							__lux_chunks.push("\n        ");
							__lux_chunks.push([
								"<div",
								__lux_attr("class", ["flex place-items-center gap-2 px-2"].join(""), false),
								">",
								function() {
									let __lux_chunks = [];
									__lux_chunks.push("\n            ");
									__lux_chunks.push([
										"<div",
										__lux_attr("class", ["flex size-4 place-items-center justify-center bg-foreground opacity-50"].join(""), false),
										">",
										function() {
											let __lux_chunks = [];
											__lux_chunks.push("\n                ");
											__lux_chunks.push(__lux_stringify(function() {
												const __lux_component = TerminalIcon;
												const __lux_component_props = { class: ["size-3 text-background"].join("") };
												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
											}()));
											__lux_chunks.push("\n            ");
											return __lux_chunks.join("");
										}(),
										"</div>"
									].join(""));
									__lux_chunks.push("\n            ");
									__lux_chunks.push(__lux_stringify(function() {
										const __lux_component = Tabs.Root;
										const __lux_component_props = {
											value: agent,
											children: function() {
												return function() {
													let __lux_chunks = [];
													__lux_chunks.push("\n                ");
													__lux_chunks.push(__lux_stringify(function() {
														const __lux_component = Tabs.List;
														const __lux_component_props = {
															class: ["h-auto bg-transparent p-0"].join(""),
															children: function() {
																return function() {
																	let __lux_chunks = [];
																	__lux_chunks.push("\n                    ");
																	__lux_chunks.push(Array.from(agents ?? []).map(function(pm) {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                        ");
																			__lux_chunks.push(__lux_stringify(function() {
																				const __lux_component = Tabs.Trigger;
																				const __lux_component_props = {
																					value: pm,
																					class: ["h-7 font-mono text-sm font-light"].join(""),
																					children: function() {
																						return function() {
																							let __lux_chunks = [];
																							__lux_chunks.push("\n                            ");
																							__lux_chunks.push(__lux_escape(__lux_stringify(pm)));
																							__lux_chunks.push("\n                        ");
																							return __lux_chunks.join("");
																						}();
																					},
																					$$slots: { default: function() {
																						return function() {
																							let __lux_chunks = [];
																							__lux_chunks.push("\n                            ");
																							__lux_chunks.push(__lux_escape(__lux_stringify(pm)));
																							__lux_chunks.push("\n                        ");
																							return __lux_chunks.join("");
																						}();
																					} }
																				};
																				return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																			}()));
																			__lux_chunks.push("\n                    ");
																			return __lux_chunks.join("");
																		}();
																	}).join(""));
																	__lux_chunks.push("\n                ");
																	return __lux_chunks.join("");
																}();
															},
															$$slots: { default: function() {
																return function() {
																	let __lux_chunks = [];
																	__lux_chunks.push("\n                    ");
																	__lux_chunks.push(Array.from(agents ?? []).map(function(pm) {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                        ");
																			__lux_chunks.push(__lux_stringify(function() {
																				const __lux_component = Tabs.Trigger;
																				const __lux_component_props = {
																					value: pm,
																					class: ["h-7 font-mono text-sm font-light"].join(""),
																					children: function() {
																						return function() {
																							let __lux_chunks = [];
																							__lux_chunks.push("\n                            ");
																							__lux_chunks.push(__lux_escape(__lux_stringify(pm)));
																							__lux_chunks.push("\n                        ");
																							return __lux_chunks.join("");
																						}();
																					},
																					$$slots: { default: function() {
																						return function() {
																							let __lux_chunks = [];
																							__lux_chunks.push("\n                            ");
																							__lux_chunks.push(__lux_escape(__lux_stringify(pm)));
																							__lux_chunks.push("\n                        ");
																							return __lux_chunks.join("");
																						}();
																					} }
																				};
																				return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																			}()));
																			__lux_chunks.push("\n                    ");
																			return __lux_chunks.join("");
																		}();
																	}).join(""));
																	__lux_chunks.push("\n                ");
																	return __lux_chunks.join("");
																}();
															} }
														};
														return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
													}()));
													__lux_chunks.push("\n            ");
													return __lux_chunks.join("");
												}();
											},
											$$slots: { default: function() {
												return function() {
													let __lux_chunks = [];
													__lux_chunks.push("\n                ");
													__lux_chunks.push(__lux_stringify(function() {
														const __lux_component = Tabs.List;
														const __lux_component_props = {
															class: ["h-auto bg-transparent p-0"].join(""),
															children: function() {
																return function() {
																	let __lux_chunks = [];
																	__lux_chunks.push("\n                    ");
																	__lux_chunks.push(Array.from(agents ?? []).map(function(pm) {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                        ");
																			__lux_chunks.push(__lux_stringify(function() {
																				const __lux_component = Tabs.Trigger;
																				const __lux_component_props = {
																					value: pm,
																					class: ["h-7 font-mono text-sm font-light"].join(""),
																					children: function() {
																						return function() {
																							let __lux_chunks = [];
																							__lux_chunks.push("\n                            ");
																							__lux_chunks.push(__lux_escape(__lux_stringify(pm)));
																							__lux_chunks.push("\n                        ");
																							return __lux_chunks.join("");
																						}();
																					},
																					$$slots: { default: function() {
																						return function() {
																							let __lux_chunks = [];
																							__lux_chunks.push("\n                            ");
																							__lux_chunks.push(__lux_escape(__lux_stringify(pm)));
																							__lux_chunks.push("\n                        ");
																							return __lux_chunks.join("");
																						}();
																					} }
																				};
																				return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																			}()));
																			__lux_chunks.push("\n                    ");
																			return __lux_chunks.join("");
																		}();
																	}).join(""));
																	__lux_chunks.push("\n                ");
																	return __lux_chunks.join("");
																}();
															},
															$$slots: { default: function() {
																return function() {
																	let __lux_chunks = [];
																	__lux_chunks.push("\n                    ");
																	__lux_chunks.push(Array.from(agents ?? []).map(function(pm) {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                        ");
																			__lux_chunks.push(__lux_stringify(function() {
																				const __lux_component = Tabs.Trigger;
																				const __lux_component_props = {
																					value: pm,
																					class: ["h-7 font-mono text-sm font-light"].join(""),
																					children: function() {
																						return function() {
																							let __lux_chunks = [];
																							__lux_chunks.push("\n                            ");
																							__lux_chunks.push(__lux_escape(__lux_stringify(pm)));
																							__lux_chunks.push("\n                        ");
																							return __lux_chunks.join("");
																						}();
																					},
																					$$slots: { default: function() {
																						return function() {
																							let __lux_chunks = [];
																							__lux_chunks.push("\n                            ");
																							__lux_chunks.push(__lux_escape(__lux_stringify(pm)));
																							__lux_chunks.push("\n                        ");
																							return __lux_chunks.join("");
																						}();
																					} }
																				};
																				return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																			}()));
																			__lux_chunks.push("\n                    ");
																			return __lux_chunks.join("");
																		}();
																	}).join(""));
																	__lux_chunks.push("\n                ");
																	return __lux_chunks.join("");
																}();
															} }
														};
														return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
													}()));
													__lux_chunks.push("\n            ");
													return __lux_chunks.join("");
												}();
											} }
										};
										return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
									}()));
									__lux_chunks.push("\n        ");
									return __lux_chunks.join("");
								}(),
								"</div>"
							].join(""));
							__lux_chunks.push("\n        ");
							__lux_chunks.push(__lux_stringify(function() {
								const __lux_component = Tooltip.Provider;
								const __lux_component_props = {
									delayDuration: 0,
									children: function() {
										return function() {
											let __lux_chunks = [];
											__lux_chunks.push("\n            ");
											__lux_chunks.push(__lux_stringify(function() {
												const __lux_component = Tooltip.Root;
												const __lux_component_props = {
													children: function() {
														return function() {
															let __lux_chunks = [];
															__lux_chunks.push("\n                ");
															__lux_chunks.push(__lux_stringify(function() {
																const __lux_component = Tooltip.Trigger;
																const __lux_component_props = {
																	children: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                    ");
																			__lux_chunks.push((_props.child = function({ props }) {
																				return function() {
																					let __lux_chunks = [];
																					__lux_chunks.push("\n                        ");
																					__lux_chunks.push(__lux_stringify(function() {
																						const __lux_component = CopyButton;
																						const __lux_component_props = {
																							...props,
																							text: commandText,
																							class: ["size-6 [&_svg]:size-3"].join(""),
																							children: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							},
																							$$slots: { default: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							} }
																						};
																						return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																					}()));
																					__lux_chunks.push("\n                    ");
																					return __lux_chunks.join("");
																				}();
																			}, ""));
																			__lux_chunks.push("\n                ");
																			return __lux_chunks.join("");
																		}();
																	},
																	$$slots: { default: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                    ");
																			__lux_chunks.push((_props.child = function({ props }) {
																				return function() {
																					let __lux_chunks = [];
																					__lux_chunks.push("\n                        ");
																					__lux_chunks.push(__lux_stringify(function() {
																						const __lux_component = CopyButton;
																						const __lux_component_props = {
																							...props,
																							text: commandText,
																							class: ["size-6 [&_svg]:size-3"].join(""),
																							children: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							},
																							$$slots: { default: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							} }
																						};
																						return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																					}()));
																					__lux_chunks.push("\n                    ");
																					return __lux_chunks.join("");
																				}();
																			}, ""));
																			__lux_chunks.push("\n                ");
																			return __lux_chunks.join("");
																		}();
																	} }
																};
																return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
															}()));
															__lux_chunks.push("\n                ");
															__lux_chunks.push(__lux_stringify(function() {
																const __lux_component = Tooltip.Content;
																const __lux_component_props = {
																	children: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("Copy to Clipboard");
																			return __lux_chunks.join("");
																		}();
																	},
																	$$slots: { default: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("Copy to Clipboard");
																			return __lux_chunks.join("");
																		}();
																	} }
																};
																return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
															}()));
															__lux_chunks.push("\n            ");
															return __lux_chunks.join("");
														}();
													},
													$$slots: { default: function() {
														return function() {
															let __lux_chunks = [];
															__lux_chunks.push("\n                ");
															__lux_chunks.push(__lux_stringify(function() {
																const __lux_component = Tooltip.Trigger;
																const __lux_component_props = {
																	children: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                    ");
																			__lux_chunks.push((_props.child = function({ props }) {
																				return function() {
																					let __lux_chunks = [];
																					__lux_chunks.push("\n                        ");
																					__lux_chunks.push(__lux_stringify(function() {
																						const __lux_component = CopyButton;
																						const __lux_component_props = {
																							...props,
																							text: commandText,
																							class: ["size-6 [&_svg]:size-3"].join(""),
																							children: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							},
																							$$slots: { default: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							} }
																						};
																						return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																					}()));
																					__lux_chunks.push("\n                    ");
																					return __lux_chunks.join("");
																				}();
																			}, ""));
																			__lux_chunks.push("\n                ");
																			return __lux_chunks.join("");
																		}();
																	},
																	$$slots: { default: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                    ");
																			__lux_chunks.push((_props.child = function({ props }) {
																				return function() {
																					let __lux_chunks = [];
																					__lux_chunks.push("\n                        ");
																					__lux_chunks.push(__lux_stringify(function() {
																						const __lux_component = CopyButton;
																						const __lux_component_props = {
																							...props,
																							text: commandText,
																							class: ["size-6 [&_svg]:size-3"].join(""),
																							children: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							},
																							$$slots: { default: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							} }
																						};
																						return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																					}()));
																					__lux_chunks.push("\n                    ");
																					return __lux_chunks.join("");
																				}();
																			}, ""));
																			__lux_chunks.push("\n                ");
																			return __lux_chunks.join("");
																		}();
																	} }
																};
																return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
															}()));
															__lux_chunks.push("\n                ");
															__lux_chunks.push(__lux_stringify(function() {
																const __lux_component = Tooltip.Content;
																const __lux_component_props = {
																	children: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("Copy to Clipboard");
																			return __lux_chunks.join("");
																		}();
																	},
																	$$slots: { default: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("Copy to Clipboard");
																			return __lux_chunks.join("");
																		}();
																	} }
																};
																return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
															}()));
															__lux_chunks.push("\n            ");
															return __lux_chunks.join("");
														}();
													} }
												};
												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
											}()));
											__lux_chunks.push("\n        ");
											return __lux_chunks.join("");
										}();
									},
									$$slots: { default: function() {
										return function() {
											let __lux_chunks = [];
											__lux_chunks.push("\n            ");
											__lux_chunks.push(__lux_stringify(function() {
												const __lux_component = Tooltip.Root;
												const __lux_component_props = {
													children: function() {
														return function() {
															let __lux_chunks = [];
															__lux_chunks.push("\n                ");
															__lux_chunks.push(__lux_stringify(function() {
																const __lux_component = Tooltip.Trigger;
																const __lux_component_props = {
																	children: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                    ");
																			__lux_chunks.push((_props.child = function({ props }) {
																				return function() {
																					let __lux_chunks = [];
																					__lux_chunks.push("\n                        ");
																					__lux_chunks.push(__lux_stringify(function() {
																						const __lux_component = CopyButton;
																						const __lux_component_props = {
																							...props,
																							text: commandText,
																							class: ["size-6 [&_svg]:size-3"].join(""),
																							children: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							},
																							$$slots: { default: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							} }
																						};
																						return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																					}()));
																					__lux_chunks.push("\n                    ");
																					return __lux_chunks.join("");
																				}();
																			}, ""));
																			__lux_chunks.push("\n                ");
																			return __lux_chunks.join("");
																		}();
																	},
																	$$slots: { default: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                    ");
																			__lux_chunks.push((_props.child = function({ props }) {
																				return function() {
																					let __lux_chunks = [];
																					__lux_chunks.push("\n                        ");
																					__lux_chunks.push(__lux_stringify(function() {
																						const __lux_component = CopyButton;
																						const __lux_component_props = {
																							...props,
																							text: commandText,
																							class: ["size-6 [&_svg]:size-3"].join(""),
																							children: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							},
																							$$slots: { default: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							} }
																						};
																						return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																					}()));
																					__lux_chunks.push("\n                    ");
																					return __lux_chunks.join("");
																				}();
																			}, ""));
																			__lux_chunks.push("\n                ");
																			return __lux_chunks.join("");
																		}();
																	} }
																};
																return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
															}()));
															__lux_chunks.push("\n                ");
															__lux_chunks.push(__lux_stringify(function() {
																const __lux_component = Tooltip.Content;
																const __lux_component_props = {
																	children: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("Copy to Clipboard");
																			return __lux_chunks.join("");
																		}();
																	},
																	$$slots: { default: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("Copy to Clipboard");
																			return __lux_chunks.join("");
																		}();
																	} }
																};
																return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
															}()));
															__lux_chunks.push("\n            ");
															return __lux_chunks.join("");
														}();
													},
													$$slots: { default: function() {
														return function() {
															let __lux_chunks = [];
															__lux_chunks.push("\n                ");
															__lux_chunks.push(__lux_stringify(function() {
																const __lux_component = Tooltip.Trigger;
																const __lux_component_props = {
																	children: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                    ");
																			__lux_chunks.push((_props.child = function({ props }) {
																				return function() {
																					let __lux_chunks = [];
																					__lux_chunks.push("\n                        ");
																					__lux_chunks.push(__lux_stringify(function() {
																						const __lux_component = CopyButton;
																						const __lux_component_props = {
																							...props,
																							text: commandText,
																							class: ["size-6 [&_svg]:size-3"].join(""),
																							children: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							},
																							$$slots: { default: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							} }
																						};
																						return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																					}()));
																					__lux_chunks.push("\n                    ");
																					return __lux_chunks.join("");
																				}();
																			}, ""));
																			__lux_chunks.push("\n                ");
																			return __lux_chunks.join("");
																		}();
																	},
																	$$slots: { default: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("\n                    ");
																			__lux_chunks.push((_props.child = function({ props }) {
																				return function() {
																					let __lux_chunks = [];
																					__lux_chunks.push("\n                        ");
																					__lux_chunks.push(__lux_stringify(function() {
																						const __lux_component = CopyButton;
																						const __lux_component_props = {
																							...props,
																							text: commandText,
																							class: ["size-6 [&_svg]:size-3"].join(""),
																							children: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							},
																							$$slots: { default: function() {
																								return function() {
																									let __lux_chunks = [];
																									__lux_chunks.push("\n                            ");
																									__lux_chunks.push((_props.icon = function() {
																										return function() {
																											let __lux_chunks = [];
																											__lux_chunks.push("\n                                ");
																											__lux_chunks.push(__lux_stringify(function() {
																												const __lux_component = ClipboardIcon;
																												const __lux_component_props = {};
																												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																											}()));
																											__lux_chunks.push("\n                            ");
																											return __lux_chunks.join("");
																										}();
																									}, ""));
																									__lux_chunks.push("\n                        ");
																									return __lux_chunks.join("");
																								}();
																							} }
																						};
																						return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
																					}()));
																					__lux_chunks.push("\n                    ");
																					return __lux_chunks.join("");
																				}();
																			}, ""));
																			__lux_chunks.push("\n                ");
																			return __lux_chunks.join("");
																		}();
																	} }
																};
																return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
															}()));
															__lux_chunks.push("\n                ");
															__lux_chunks.push(__lux_stringify(function() {
																const __lux_component = Tooltip.Content;
																const __lux_component_props = {
																	children: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("Copy to Clipboard");
																			return __lux_chunks.join("");
																		}();
																	},
																	$$slots: { default: function() {
																		return function() {
																			let __lux_chunks = [];
																			__lux_chunks.push("Copy to Clipboard");
																			return __lux_chunks.join("");
																		}();
																	} }
																};
																return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
															}()));
															__lux_chunks.push("\n            ");
															return __lux_chunks.join("");
														}();
													} }
												};
												return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
											}()));
											__lux_chunks.push("\n        ");
											return __lux_chunks.join("");
										}();
									} }
								};
								return __lux_component && typeof __lux_component.render === "function" ? __lux_component.render(__lux_component_props) : typeof __lux_component === "function" ? __lux_component(__lux_component_props) : "";
							}()));
							__lux_chunks.push("\n    ");
							return __lux_chunks.join("");
						}(),
						"</div>"
					].join(""));
					__lux_chunks.push("\n    ");
					__lux_chunks.push([
						"<div",
						__lux_attr("class", ["no-scrollbar overflow-x-auto p-3"].join(""), false),
						">",
						function() {
							let __lux_chunks = [];
							__lux_chunks.push("\n		");
							__lux_chunks.push([
								"<span",
								__lux_attr("class", ["font-mono text-sm leading-none font-light text-nowrap text-muted-foreground"].join(""), false),
								">",
								function() {
									let __lux_chunks = [];
									__lux_chunks.push("\n			");
									__lux_chunks.push(__lux_escape(__lux_stringify(commandText)));
									__lux_chunks.push("\n		");
									return __lux_chunks.join("");
								}(),
								"</span>"
							].join(""));
							__lux_chunks.push("\n    ");
							return __lux_chunks.join("");
						}(),
						"</div>"
					].join(""));
					__lux_chunks.push("\n");
					return __lux_chunks.join("");
				}(),
				"</div>"
			].join(""));
			__lux_chunks.push("\n\n");
			__lux_chunks.push("\n");
			return __lux_chunks.join("");
		}();
	}
};
