# SHD paired input-rate control

**Protocol:** v143  
**Hash:** `shd-0c1-98d17bee8a5e706e`  
**Comparison:** full-superspike  
**Schedule:** 24/8, 1 epochs, lr 0.010, 1 seeds  
**Data:** fixture / non-citable  
**Verdict:** **PILOT**

Input-only accuracy: 0.2500. Hidden accuracy: 0.1250.  
Hidden − input-only: mean -0.1250, hierarchical-bootstrap 95% CI [-0.3750, 0.0000].  
Shuffled-label input control: 0.2500. Input degenerate: yes. Hidden degenerate: yes.  
No test-time updates and deterministic prediction replay: **yes**.

Equivalence requires mean < 0.02 and upper 95% bound < 0.05. The unregistered pilot is not used in this verdict.
