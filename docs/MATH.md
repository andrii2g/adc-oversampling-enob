# Mathematical Contract

## ADC quantization

For an unsigned N-bit ADC:

```text
levels = 2^N
max_code = 2^N - 1
LSB = Vref / max_code
```

For analog voltage `v`:

```text
v_clamped = clamp(v, 0, Vref)
code = round(v_clamped / Vref * max_code)
v_adc = code / max_code * Vref
```

## Dither/noise

Noise is added before quantization:

```text
v_input = v_true + n
code = Q(v_input)
```

Uniform mode with configured amplitude A LSB:

```text
n ~ U(-A*LSB, +A*LSB)
```

Gaussian mode:

```text
n ~ Normal(0, (A*LSB)^2)
```

## Oversampling and averaging

For independent samples with noise variance `sigma^2`, the mean of M samples has:

```text
Var(mean) = sigma^2 / M
StdDev(mean) = sigma / sqrt(M)
```

The improvement in bits from a reduction in random error standard deviation is:

```text
DeltaBits = log2(sigma_1 / sigma_M)
```

Under the ideal independent-noise model:

```text
DeltaBits = log2(sqrt(M))
          = 0.5 * log2(M)
```

Hence:

| OSR M | Ideal gain |
|---:|---:|
| 1 | 0 bits |
| 4 | 1 bit |
| 16 | 2 bits |
| 64 | 3 bits |
| 256 | 4 bits |

Equivalently, approximately `4^n` samples are needed for `n` additional bits.

## Error metrics

For error:

```text
e_i = measured_i - reference_i
```

Bias:

```text
bias = mean(e_i)
```

Population variance:

```text
variance = mean((e_i - bias)^2)
```

Standard deviation:

```text
stddev = sqrt(variance)
```

RMSE:

```text
RMSE = sqrt(mean(e_i^2))
```

For population statistics:

```text
RMSE^2 = variance + bias^2
```

This distinction is central: averaging can reduce random spread while leaving a systematic quantization bias unchanged.

## SNR used by the simulator

For non-constant reference data:

```text
reference_ac_i = reference_i - mean(reference)
signal_rms = sqrt(mean(reference_ac_i^2))
error_rms = sqrt(mean(error_i^2))
SNR_dB = 20 * log10(signal_rms / error_rms)
```

For a constant signal, `signal_rms` after DC removal is zero, so conventional SNR is reported as N/A.

## SNR-derived ENOB

The familiar ideal ADC relation:

```text
SNR_dB ~= 6.02*N + 1.76
```

gives:

```text
ENOB = (SNR_dB - 1.76) / 6.02
```

This relation is conventionally associated with a full-scale sinusoidal input and ideal quantization assumptions. The simulator reports the transformed value for nonconstant ramp references when SNR is defined. It is a comparative metric, not a standards-grade ADC dynamic-performance measurement.

## Relative effective-resolution gain

For the constant/dither experiment, use a baseline-relative metric:

```text
RMSE_gain_bits(M) = log2(RMSE_1 / RMSE_M)
StdDev_gain_bits(M) = log2(StdDev_1 / StdDev_M)
```

`StdDev_gain_bits` follows the independent random-noise theory most directly.

`RMSE_gain_bits` also includes bias and therefore may plateau if systematic quantization offset remains.

## Why no-dither oversampling can fail

For a constant analog input that always maps to code C:

```text
C, C, C, C, ...
```

Every block average remains exactly C regardless of M. More samples contain no new threshold-crossing information.

When dither makes adjacent codes occur with probabilities related to the fractional analog position, the average code can converge to a fractional value.

## Conditions for useful oversampling

The ideal gain assumes approximately:

- stationary noise statistics;
- sufficiently independent or weakly correlated samples;
- enough noise/dither to exercise adjacent quantization levels;
- no clipping-dominated regime;
- a stable underlying signal over the averaging window.

Correlated noise breaks the simple `1/sqrt(M)` reduction. Large dither can increase absolute error even though averaging still reduces its variance.

## Resolution is not accuracy

Oversampling cannot repair systematic hardware errors such as:

- bad calibration;
- reference-voltage error;
- offset;
- gain error;
- INL/DNL;
- sensor bias.

A fractional averaged code means additional precision/information under a model, not guaranteed absolute accuracy.
