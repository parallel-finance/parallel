(function() {var implementors = {};
implementors["pallet_traits"] = [{"text":"impl&lt;AccountId&gt; Convert&lt;AccountId, MultiLocation&gt; for <a class=\"struct\" href=\"pallet_traits/xcm/struct.AccountIdToMultiLocation.html\" title=\"struct pallet_traits::xcm::AccountIdToMultiLocation\">AccountIdToMultiLocation</a>&lt;AccountId&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;AccountId: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.array.html\">[</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.array.html\">; 32]</a>&gt;,&nbsp;</span>","synthetic":false,"types":["pallet_traits::xcm::AccountIdToMultiLocation"]},{"text":"impl&lt;LegacyAssetConverter, ForeignAssetConverter&gt; Convert&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u32.html\">u32</a>, <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;MultiLocation&gt;&gt; for <a class=\"struct\" href=\"pallet_traits/xcm/struct.CurrencyIdtoMultiLocation.html\" title=\"struct pallet_traits::xcm::CurrencyIdtoMultiLocation\">CurrencyIdtoMultiLocation</a>&lt;LegacyAssetConverter, ForeignAssetConverter&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;LegacyAssetConverter: Convert&lt;<a class=\"type\" href=\"parallel_primitives/type.CurrencyId.html\" title=\"type parallel_primitives::CurrencyId\">CurrencyId</a>, <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;MultiLocation&gt;&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;ForeignAssetConverter: Convert&lt;MultiLocation, <a class=\"type\" href=\"parallel_primitives/type.CurrencyId.html\" title=\"type parallel_primitives::CurrencyId\">CurrencyId</a>&gt;,&nbsp;</span>","synthetic":false,"types":["pallet_traits::xcm::CurrencyIdtoMultiLocation"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()