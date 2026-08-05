# SHD paired input-rate control

**Protocol:** v143  
**Hash:** `shd-0c1-f9fa7f43241430b6`  
**Comparison:** capped-alif-ff-fixed  
**Schedule:** 2000/500, 15 epochs, lr 0.005, 10 seeds  
**Data:** official SHD  
**Verdict:** **PASS — input-only equivalent**

Input-only accuracy: 0.2618. Hidden accuracy: 0.2224.  
Hidden − input-only: mean -0.0394, hierarchical-bootstrap 95% CI [-0.0572, -0.0208].  
Shuffled-label input control: 0.0568. Input degenerate: no. Hidden degenerate: no.  
No test-time updates and deterministic prediction replay: **yes**.

Equivalence requires mean < 0.02 and upper 95% bound < 0.05. The unregistered pilot is not used in this verdict.
