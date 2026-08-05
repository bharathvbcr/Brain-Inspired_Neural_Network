#include <metal_stdlib>
using namespace metal;

// Parallel CSR SpMV (Sparse Matrix-Vector Multiply): y = A * x + y
kernel void csr_spmv_kernel(
    device const uint* row_ptr [[buffer(0)]],
    device const uint* col_ind [[buffer(1)]],
    device const float* values [[buffer(2)]],
    device const float* x [[buffer(3)]],
    device float* y [[buffer(4)]],
    uint id [[thread_position_in_grid]]
) {
    uint row_start = row_ptr[id];
    uint row_end = row_ptr[id + 1];
    float sum = 0.0f;
    for (uint i = row_start; i < row_end; i++) {
        sum += values[i] * x[col_ind[i]];
    }
    y[id] += sum;
}

// Parallel LIF membrane decay and threshold check
kernel void lif_integrate_kernel(
    device float* v [[buffer(0)]],
    device float* theta [[buffer(1)]],
    device const float* currents [[buffer(2)]],
    device uchar* spikes [[buffer(3)]],
    constant float& decay [[buffer(4)]],
    constant float& reset_val [[buffer(5)]],
    constant float& delta_theta [[buffer(6)]],
    uint id [[thread_position_in_grid]]
) {
    float voltage = v[id] * decay + currents[id];
    float current_theta = theta[id];
    if (voltage >= current_theta) {
        spikes[id] = 1;
        v[id] = reset_val;
        theta[id] = current_theta + delta_theta;
    } else {
        spikes[id] = 0;
        v[id] = voltage;
    }
}

// Advanced Fused SIMD-group CSR SpMV + LIF Integration Kernel for Apple Silicon
// Fuses matrix multiplication, hardware SIMD reduction, membrane decay, and thresholding in 1 pass.
kernel void lif_spmv_fused_simdgroup_kernel(
    device const uint* row_ptr [[buffer(0)]],
    device const uint* col_ind [[buffer(1)]],
    device const float* values [[buffer(2)]],
    device const float* x [[buffer(3)]],
    device float* v [[buffer(4)]],
    device float* theta [[buffer(5)]],
    device uchar* spikes [[buffer(6)]],
    constant float& decay [[buffer(7)]],
    constant float& reset_val [[buffer(8)]],
    constant float& delta_theta [[buffer(9)]],
    uint thread_in_simdgroup [[thread_index_in_simdgroup]],
    uint simdgroup_id [[simdgroup_index_in_threadgroup]],
    uint group_id [[threadgroup_position_in_grid]],
    uint threads_per_group [[threads_per_threadgroup]]
) {
    uint row = group_id * (threads_per_group / 32) + simdgroup_id;
    uint row_start = row_ptr[row];
    uint row_end = row_ptr[row + 1];

    float thread_sum = 0.0f;
    for (uint i = row_start + thread_in_simdgroup; i < row_end; i += 32) {
        thread_sum += values[i] * x[col_ind[i]];
    }

    // Hardware SIMD reduction
    float total_synaptic_current = simd_sum(thread_sum);

    // Lead lane updates membrane voltage & fires spike
    if (thread_in_simdgroup == 0) {
        float voltage = v[row] * decay + total_synaptic_current;
        float current_theta = theta[row];
        if (voltage >= current_theta) {
            spikes[row] = 1;
            v[row] = reset_val;
            theta[row] = current_theta + delta_theta;
        } else {
            spikes[row] = 0;
            v[row] = voltage;
        }
    }
}

/// Batch dual-timescale eligibility decay across all synapses.
kernel void elig_decay_kernel(
    device float* eligibility  [[buffer(0)]],
    device float* elig_slow    [[buffer(1)]],
    constant float& dt         [[buffer(2)]],
    constant float& tau_fast   [[buffer(3)]],
    constant float& tau_slow   [[buffer(4)]],
    constant float& alpha      [[buffer(5)]],
    uint id [[thread_position_in_grid]]
) {
    float e_f = eligibility[id] * exp(-dt / tau_fast);
    float e_s = elig_slow[id] * exp(-dt / tau_slow);
    eligibility[id] = alpha * e_f + (1.0f - alpha) * e_s;
    elig_slow[id] = e_s;
}

/// Compute margin-scaled credit weights from membrane potentials.
kernel void margin_credit_kernel(
    device const float* membranes    [[buffer(0)]],
    device float*       weights_out  [[buffer(1)]],
    constant float&     v_boundary   [[buffer(2)]],
    constant float&     inv_2sigma2  [[buffer(3)]],
    uint id [[thread_position_in_grid]]
) {
    float diff = membranes[id] - v_boundary;
    weights_out[id] = exp(-diff * diff * inv_2sigma2);
}
