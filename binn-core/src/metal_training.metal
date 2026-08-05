#include <metal_stdlib>
using namespace metal;

/// Fused single-dispatch GPU training step:
/// 1. SpMV synaptic current accumulation
/// 2. LIF membrane integrate & spike generation
/// 3. Dual-timescale eligibility trace decay
/// 4. Winner-margin credit scaling
/// 5. In-place weight update (w += eta * e * M - lambda * w)
kernel void fused_training_step_kernel(
    device const uint*   row_ptr        [[buffer(0)]],
    device const uint*   col_ind        [[buffer(1)]],
    device float*        weights        [[buffer(2)]],
    device const float*  x              [[buffer(3)]],
    device float*        v              [[buffer(4)]],
    device float*        theta          [[buffer(5)]],
    device uchar*        spikes         [[buffer(6)]],
    device float*        eligibility    [[buffer(7)]],
    device float*        elig_slow      [[buffer(8)]],
    constant float&      decay          [[buffer(9)]],
    constant float&      v_reset        [[buffer(10)]],
    constant float&      delta_theta    [[buffer(11)]],
    constant float&      dt             [[buffer(12)]],
    constant float&      tau_fast       [[buffer(13)]],
    constant float&      tau_slow       [[buffer(14)]],
    constant float&      alpha          [[buffer(15)]],
    constant float&      v_boundary     [[buffer(16)]],
    constant float&      inv_2sigma2    [[buffer(17)]],
    constant float&      eta            [[buffer(18)]],
    constant float&      lambda_val     [[buffer(19)]],
    constant float&      credit_signal  [[buffer(20)]],
    uint id [[thread_position_in_grid]]
) {
    // 1. SpMV current
    float current = 0.0f;
    uint start = row_ptr[id];
    uint end = row_ptr[id + 1];
    for (uint i = start; i < end; ++i) {
        current += weights[i] * x[col_ind[i]];
    }

    // 2. LIF Integrate
    float membrane = v[id] * decay + current;
    float th = theta[id];
    uchar s = 0;
    if (membrane >= th) {
        s = 1;
        membrane = v_reset;
        th += delta_theta;
    }
    v[id] = membrane;
    theta[id] = th;
    spikes[id] = s;

    // 3. Margin Credit
    float diff = membrane - v_boundary;
    float margin_w = exp(-diff * diff * inv_2sigma2);
    float effective_credit = credit_signal * margin_w;

    // 4. Dual Eligibility Decay & Weight Update for incoming synapses
    for (uint i = start; i < end; ++i) {
        float e_f = eligibility[i] * exp(-dt / tau_fast);
        float e_s = elig_slow[i] * exp(-dt / tau_slow);
        float e_comb = alpha * e_f + (1.0f - alpha) * e_s;
        eligibility[i] = e_comb;
        elig_slow[i] = e_s;

        // Weight update
        float dw = eta * e_comb * effective_credit - lambda_val * weights[i];
        weights[i] += dw;
    }
}
