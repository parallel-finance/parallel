# Stable Swap

StableSwap implementation for Parallel.fi

## Functionality
* To calculate delta simply call :- `Self::delta_util(tot_base_amount, tot_quote_amount).unwrap()`

## References 
* https://curve.fi/files/stableswap-paper.pdf
* https://github.com/equilibrium-eosdt/equilibrium-curve-amm/blob/master/docs/deducing-get_y-formulas.pdf
* https://miguelmota.com/blog/understanding-stableswap-curve/
* https://github.com/curvefi/curve-contract/blob/master/contracts/pool-templates/base/SwapTemplateBase.vy
