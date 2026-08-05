# SHD paired input-rate control

**Protocol:** v143  
**Hash:** `shd-0c1-cd73ab9214c962e7`  
**Comparison:** full-superspike  
**Schedule:** 8156/2264, 20 epochs, lr 0.020, 10 seeds  
**Data:** official SHD  
**Verdict:** **PASS — input-only equivalent**

Input-only accuracy: 0.4428. Hidden accuracy: 0.4157.  
Hidden − input-only: mean -0.0270, hierarchical-bootstrap 95% CI [-0.0432, -0.0087].  
Shuffled-label input control: 0.0468. Input degenerate: no. Hidden degenerate: no.  
No test-time updates and deterministic prediction replay: **yes**.

Equivalence requires mean < 0.02 and upper 95% bound < 0.05. The unregistered pilot is not used in this verdict.
