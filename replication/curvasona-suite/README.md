# CURVASONA's 40-curve suite

> **Generated** by `cargo xtask replication` from `sonarithm_curvasona::suite`; do not edit by hand.

Not a published design: this group renders the 40 curves whose critical points are known analytically, the suite CURVASONA is tested against (07 CURVASONA, C03-T02 and C03-T03). Each spec plays its curve as one 8-second glide, y as pitch with the default range, and its critical points as earcons (`analysis.detect: ["critical-points"]`, RFC-0007 §2). The list below is what CURVASONA finds at the default 1000 samples, the text equivalent of each render's cues. Horizontal asymptotes are listed but have no earcon.

| Spec | f(x) | Domain | Critical points |
|---|---|---|---|
| [01-parabola](01-parabola.json) | `x^2` | [-3, 3] | zero at 0; local minimum at 0 |
| [02-cubic](02-cubic.json) | `x^3 - 3*x` | [-3, 3] | zero at -1.7321; local maximum at -1; zero at 0; inflection point at 0; local minimum at 1; zero at 1.7321 |
| [03-sine](03-sine.json) | `sin(x)` | [-10, 10] | zero at -9.4248; inflection point at -9.4248; local minimum at -7.854; inflection point at -6.2832; zero at -6.2832; local maximum at -4.7124; inflection point at -3.1416; zero at -3.1416; local minimum at -1.5708; zero at 0; inflection point at 0; local maximum at 1.5708; zero at 3.1416; inflection point at 3.1416; local minimum at 4.7124; zero at 6.2832; inflection point at 6.2832; local maximum at 7.854; inflection point at 9.4248; zero at 9.4248 |
| [04-cosine](04-cosine.json) | `cos(x)` | [-10, 10] | local minimum at -9.4248; zero at -7.854; inflection point at -7.854; local maximum at -6.2832; inflection point at -4.7124; zero at -4.7124; local minimum at -3.1416; inflection point at -1.5708; zero at -1.5708; local maximum at 0; zero at 1.5708; inflection point at 1.5708; local minimum at 3.1416; zero at 4.7124; inflection point at 4.7124; local maximum at 6.2832; inflection point at 7.854; zero at 7.854; local minimum at 9.4248 |
| [05-tangent](05-tangent.json) | `tan(x)` | [-5, 5] | vertical asymptote at -4.7124; inflection point at -3.1416; zero at -3.1416; vertical asymptote at -1.5708; zero at 0; inflection point at 0; vertical asymptote at 1.5708; zero at 3.1416; inflection point at 3.1416; vertical asymptote at 4.7124 |
| [06-reciprocal](06-reciprocal.json) | `1/x` | [-5, 5] | horizontal asymptote to the left, y = 0; vertical asymptote at 0; horizontal asymptote to the right, y = 0 |
| [07-shifted-reciprocal](07-shifted-reciprocal.json) | `1/(x - 1)` | [-4, 4] | horizontal asymptote to the left, y = 0; vertical asymptote at 1; horizontal asymptote to the right, y = 0 |
| [08-inverse-square](08-inverse-square.json) | `1/x^2` | [-3, 3] | horizontal asymptote to the left, y = 0; vertical asymptote at 0; horizontal asymptote to the right, y = 0 |
| [09-line-with-a-hole](09-line-with-a-hole.json) | `(x^2 - 1)/(x - 1)` | [-3, 3] | zero at -1; hole at 1 |
| [10-sinc](10-sinc.json) | `sin(x)/x` | [-20, 20] | horizontal asymptote to the left, y = 0; zero at -18.8496; inflection point at -18.7426; local minimum at -17.2208; zero at -15.708; inflection point at -15.5792; local maximum at -14.0662; zero at -12.5664; inflection point at -12.4044; local minimum at -10.9041; zero at -9.4248; inflection point at -9.2058; local maximum at -7.7253; zero at -6.2832; inflection point at -5.9404; local minimum at -4.4934; zero at -3.1416; inflection point at -2.0816; hole at 0; inflection point at 2.0816; zero at 3.1416; local minimum at 4.4934; inflection point at 5.9404; zero at 6.2832; local maximum at 7.7253; inflection point at 9.2058; zero at 9.4248; local minimum at 10.9041; inflection point at 12.4044; zero at 12.5664; local maximum at 14.0662; inflection point at 15.5792; zero at 15.708; local minimum at 17.2208; inflection point at 18.7426; zero at 18.8496; horizontal asymptote to the right, y = 0 |
| [11-natural-logarithm](11-natural-logarithm.json) | `ln(x)` | [-2, 5] | undefined from -2 to 0; vertical asymptote at 0; zero at 1 |
| [12-square-root](12-square-root.json) | `sqrt(x)` | [-4, 4] | undefined from -4 to 0; zero at 0 |
| [13-semicircle](13-semicircle.json) | `sqrt(1 - x^2)` | [-2, 2] | undefined from -2 to -1; zero at -1; local maximum at 0; zero at 1; undefined from 1 to 2 |
| [14-narrow-gap](14-narrow-gap.json) | `sqrt(x^2 - 0.000001)` | [-1, 1] | zero at -0.001; undefined from -0.001 to 0.001; zero at 0.001 |
| [15-two-poles](15-two-poles.json) | `1/(x^2 - 4)` | [-5, 5] | horizontal asymptote to the left, y = 0; vertical asymptote at -2; local maximum at 0; vertical asymptote at 2; horizontal asymptote to the right, y = 0 |
| [16-bounded-rational](16-bounded-rational.json) | `x/(x^2 + 1)` | [-5, 5] | horizontal asymptote to the left, y = 0; inflection point at -1.7321; local minimum at -1; zero at 0; inflection point at 0; local maximum at 1; inflection point at 1.7321; horizontal asymptote to the right, y = 0 |
| [17-exponential](17-exponential.json) | `exp(x)` | [-3, 3] | horizontal asymptote to the left, y = 0 |
| [18-bell](18-bell.json) | `exp(-x^2)` | [-3, 3] | horizontal asymptote to the left, y = 0; inflection point at -0.7071; local maximum at 0; inflection point at 0.7071; horizontal asymptote to the right, y = 0 |
| [19-floor](19-floor.json) | `floor(x)` | [-2.5, 2.5] | jump at -2; jump at -1; jump at 0; jump at 1; jump at 2 |
| [20-sign-by-division](20-sign-by-division.json) | `x/abs(x)` | [-2, 2] | jump at 0 |
| [21-absolute-value](21-absolute-value.json) | `abs(x)` | [-2, 2] | zero at 0; local minimum at 0 |
| [22-secant](22-secant.json) | `sec(x)` | [-4, 4] | local maximum at -3.1416; vertical asymptote at -1.5708; local minimum at 0; vertical asymptote at 1.5708; local maximum at 3.1416 |
| [23-cotangent](23-cotangent.json) | `cot(x)` | [-3, 3] | inflection point at -1.5708; zero at -1.5708; vertical asymptote at 0; zero at 1.5708; inflection point at 1.5708 |
| [24-cosecant](24-cosecant.json) | `csc(x)` | [0.5, 6] | local minimum at 1.5708; vertical asymptote at 3.1416; local maximum at 4.7124 |
| [25-inverse-hyperbolic-tangent](25-inverse-hyperbolic-tangent.json) | `atanh(x)` | [-2, 2] | undefined from -2 to -1; vertical asymptote at -1; zero at 0; inflection point at 0; vertical asymptote at 1; undefined from 1 to 2 |
| [26-arcsine](26-arcsine.json) | `asin(x)` | [-2, 2] | undefined from -2 to -1; zero at 0; inflection point at 0; undefined from 1 to 2 |
| [27-log-of-the-absolute-value](27-log-of-the-absolute-value.json) | `ln(abs(x))` | [-3, 3] | zero at -1; vertical asymptote at 0; zero at 1 |
| [28-cubic-over-a-line](28-cubic-over-a-line.json) | `(x^3 - 8)/(x - 2)` | [0, 4] | hole at 2 |
| [29-logistic](29-logistic.json) | `1/(1 + exp(-x))` | [-6, 6] | horizontal asymptote to the left, y = 0; inflection point at 0; horizontal asymptote to the right, y = 1 |
| [30-fractional-power](30-fractional-power.json) | `x^(1/3)` | [-8, 8] | undefined from -8 to 0; zero at 0 |
| [31-cube-root](31-cube-root.json) | `cbrt(x)` | [-8, 8] | zero at 0; inflection point at 0 |
| [32-hole-and-pole](32-hole-and-pole.json) | `(x + 1)/(x^2 - 1)` | [-3, 3] | horizontal asymptote to the left, y = 0; hole at -1; vertical asymptote at 1; horizontal asymptote to the right, y = 0 |
| [33-round](33-round.json) | `round(x)` | [-2, 2] | jump at -1.5; jump at -0.5; jump at 0.5; jump at 1.5 |
| [34-two-linear-poles](34-two-linear-poles.json) | `1/((x - 1)*(x + 2))` | [-4, 4] | horizontal asymptote to the left, y = 0; vertical asymptote at -2; local maximum at -0.5; vertical asymptote at 1; horizontal asymptote to the right, y = 0 |
| [35-one-sided-pole](35-one-sided-pole.json) | `exp(1/x)` | [-2, 2] | horizontal asymptote to the left, y = 1; inflection point at -0.5; vertical asymptote at 0; horizontal asymptote to the right, y = 1 |
| [36-half-angle-tangent](36-half-angle-tangent.json) | `tan(x/2)` | [-10, 10] | vertical asymptote at -9.4248; inflection point at -6.2832; zero at -6.2832; vertical asymptote at -3.1416; zero at 0; inflection point at 0; vertical asymptote at 3.1416; zero at 6.2832; inflection point at 6.2832; vertical asymptote at 9.4248 |
| [37-oblique-asymptote](37-oblique-asymptote.json) | `x^2/(x - 3)` | [0, 5] | zero at 0; vertical asymptote at 3 |
| [38-log-of-a-parabola](38-log-of-a-parabola.json) | `ln(x^2 - 1)` | [-3, 3] | zero at -1.4142; vertical asymptote at -1; undefined from -1 to 1; vertical asymptote at 1; zero at 1.4142 |
| [39-shifted-sinc](39-shifted-sinc.json) | `sin(x)/(x - pi)` | [0, 6] | zero at 0; horizontal asymptote to the left, y = 0; inflection point at 1.06; hole at 3.1416; inflection point at 5.2232; horizontal asymptote to the right, y = 0 |
| [40-arccosine](40-arccosine.json) | `acos(x/2)` | [-3, 3] | undefined from -3 to -2; inflection point at 0; zero at 2; undefined from 2 to 3 |

The reproducible part of each render manifest is in [manifests.json](manifests.json). Specs and records are CC BY 4.0, like the rest of the gallery.
