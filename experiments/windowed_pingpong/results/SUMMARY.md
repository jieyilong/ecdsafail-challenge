# Experiment 2 results

All means use the complete shared 100,000-input corpus, including failed inputs.

| Circuit | Q | Mean T | Static T | Mean counted Clifford | QT | Any failures | Pass fraction | QT / pass fraction |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| mixed_pingpong | 1,321 | 952,719.42315 | 1,009,077 | 11,110,772.13321 | 1,258,542,357.98115 | 42 | 0.99958 | 1,259,071,167.87166 |
| windowed_pingpong | 1,338 | 1,288,559.92169 | 1,345,161 | 96,480,649.46440 | 1,724,093,175.22122 | 36 | 0.99964 | 1,724,714,072.28724 |
| mixed_jump2 | 1,151 | 1,299,440.79269 | 1,358,316 | 5,346,775.58834 | 1,495,656,352.38619 | 192 | 0.99808 | 1,498,533,536.77680 |
| windowed_jump2 | 1,162 | 1,684,160.92396 | 1,754,382 | 90,754,194.45465 | 1,956,994,993.64152 | 191 | 0.99809 | 1,960,740,007.05500 |

The retry-adjusted value is a per-call sensitivity proxy, not the cost or success probability of a coherent Shor computation.
