# Shared by the icon generators: how an icon name becomes a Rust enum variant.

# `house-line` -> `HouseLine`, `arrow_back` -> `ArrowBack`, `10k` -> `N10k` (a variant cannot
# start with a digit).
export def pascal [name: string] {
  let p = ($name | split row -r '[-_]' | where {|t| $t != ""}
    | each {|t| ($t | str substring 0..0 | str uppercase) + ($t | str substring 1..) }
    | str join)
  if ($p =~ '^[0-9]') { $"N($p)" } else { $p }
}

# Normalised key: Cyberpunk, Phosphor and Material spell the same name with `-` or `_`.
export def key [name: string] { $name | str replace -a '-' '_' }
