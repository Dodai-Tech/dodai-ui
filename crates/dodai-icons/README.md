# dodai-icons

Framework-agnostic icon geometry for Dodai UI. Each `IconName` yields stroked
polylines on a 24×24 grid (`IconName::shapes()`); adapters (e.g. `dodai-gpui`'s
`Icon`) scale and stroke them in the current theme colour. No assets, no deps.
