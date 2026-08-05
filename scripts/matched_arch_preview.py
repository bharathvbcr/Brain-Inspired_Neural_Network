import numpy as np

T = 8
N_IN = 2
TAU_M = 20.0
ALPHA = np.exp(-1.0/TAU_M)
THETA = 1.0
VRESET = 0.0
BETA = 5.0

def surrogate(u_minus_theta, beta=BETA):
    d = 1.0 + beta*np.abs(u_minus_theta)
    return 1.0/(d*d)

def sigmoid(z): return 1.0/(1.0+np.exp(-z))

def gen_examples(n, seed):
    rng = np.random.default_rng(seed)
    X1 = np.zeros((n,T)); X2 = np.zeros((n,T)); Y=np.zeros(n)
    for i in range(n):
        t1 = rng.integers(T); X1[i,t1]=1.0
        if rng.random()<0.5:
            # coincident within +-1
            opts=[t for t in (t1-1,t1,t1+1) if 0<=t<T]
            t2=rng.choice(opts)
        else:
            t2=rng.integers(T)
            while abs(t2-t1)<=1: t2=rng.integers(T)
        X2[i,t2]=1.0
        Y[i]=1.0 if abs(t1-t2)<=1 else 0.0
    return X1,X2,Y

class Arch:
    def __init__(self,h,seed):
        rng=np.random.default_rng(seed^0x5171)
        self.h=h
        self.win=(rng.random((h,N_IN))*2-1)*0.5
        self.wrec=(rng.random((h,h))*2-1)*(0.3/np.sqrt(h))
        self.wout=(rng.random(h)*2-1)*0.2
        self.by=0.0
    def forward(self,x1,x2):
        h=self.h
        u=np.zeros((h,T)); s=np.zeros((h,T))
        x=np.stack([x1,x2],axis=1) # T x 2? -> build per t
        for t in range(T):
            cur=self.win[:,0]*x1[t]+self.win[:,1]*x2[t]
            if t>0: cur=cur+self.wrec@s[:,t-1]
            uprev=u[:,t-1] if t>0 else np.full(h,VRESET)
            sprev=s[:,t-1] if t>0 else np.zeros(h)
            ui=ALPHA*uprev+cur-THETA*sprev
            u[:,t]=ui
            s[:,t]=(ui>=THETA).astype(float)
        rates=s.sum(axis=1)
        logit=self.by+self.wout@rates
        return dict(u=u,s=s,rates=rates,logit=logit,x1=x1,x2=x2)

def eval_acc(arch,X1,X2,Y):
    c=0
    for i in range(len(Y)):
        p=sigmoid(arch.forward(X1[i],X2[i])['logit'])
        if abs((1.0 if p>=0.5 else 0.0)-Y[i])<0.5: c+=1
    return c/len(Y)

def train_gradient(arch,epochs,lr,X1,X2,Y):
    h=arch.h
    for _ in range(epochs):
        for i in range(len(Y)):
            c=arch.forward(X1[i],X2[i])
            dlogit=sigmoid(c['logit'])-Y[i]
            dwout=dlogit*c['rates']
            g_r=dlogit*arch.wout
            du_next=np.zeros(h)
            dwin=np.zeros((h,N_IN)); dwrec=np.zeros((h,h))
            for t in range(T-1,-1,-1):
                ds=g_r - du_next + arch.wrec.T@du_next
                surr=surrogate(c['u'][:,t]-THETA)
                du=ds*surr+ALPHA*du_next
                dwin[:,0]+=du*c['x1'][t]; dwin[:,1]+=du*c['x2'][t]
                if t>0: dwrec+=np.outer(du,c['s'][:,t-1])
                du_next=du
            arch.win-=lr*dwin; arch.wrec-=lr*dwrec
            arch.wout-=lr*dwout; arch.by-=lr*dlogit

def train_local(arch,epochs,eta,lam,X1,X2,Y,seed):
    h=arch.h; rng=np.random.default_rng(seed^0x3FAC70)
    for _ in range(epochs):
        for i in range(len(Y)):
            c=arch.forward(X1[i],X2[i])
            p=sigmoid(c['logit'])
            a=1.0 if rng.random()<p else 0.0
            reward=1.0 if abs(a-Y[i])<0.5 else -1.0
            m=reward
            # hidden eligibility
            e_in=np.zeros((h,N_IN)); e_rec=np.zeros((h,h))
            ei0=np.zeros(h); ei1=np.zeros(h); erow=np.zeros((h,h))
            for t in range(T):
                surr=surrogate(c['u'][:,t]-THETA)
                ei0=ALPHA*ei0+surr*c['x1'][t]
                ei1=ALPHA*ei1+surr*c['x2'][t]
                if t>0: erow=ALPHA*erow+np.outer(surr,c['s'][:,t-1])
                else: erow=ALPHA*erow
            e_in[:,0]=ei0; e_in[:,1]=ei1; e_rec=erow
            eout_scale=a-p
            arch.wout+=eta*m*(eout_scale*c['rates'])-lam*arch.wout
            arch.by+=eta*m*eout_scale
            arch.win+=eta*m*e_in-lam*arch.win
            arch.wrec+=eta*m*e_rec-lam*arch.wrec

def run(seed, h=128, epochs=80, n_train=80, n_test=40):
    X1,X2,Y=gen_examples(n_train,seed^0xA1)
    TX1,TX2,TY=gen_examples(n_test,seed^0xB2)
    g=Arch(h,seed); train_gradient(g,epochs,0.05,X1,X2,Y); ga=eval_acc(g,TX1,TX2,TY)
    l=Arch(h,seed); train_local(l,epochs,0.35,0.002,X1,X2,Y,seed); la=eval_acc(l,TX1,TX2,TY)
    return ga,la

seeds=range(8)  # preview (full Rust run is n=20)
gas=[]; las=[]; gaps=[]
for s in seeds:
    ga,la=run(s)
    gas.append(ga); las.append(la)
    denom=max(ga-0.5,1e-6)
    gap=np.clip((la-0.5)/denom,0,1) if ga-0.5>=0.15 else 0.0
    gaps.append(gap)
    print(f"seed {s}: gradient={ga:.3f}  local={la:.3f}  gap_closed={gap:.3f}")

gas=np.array(gas); las=np.array(las); gaps=np.array(gaps)
n=len(gaps)
mean_gap=gaps.mean(); lcb=mean_gap-1.96*gaps.std(ddof=1)/np.sqrt(n)
print("\n=== matched-arch preview (n=%d seeds) ==="%n)
print(f"matched-gradient (ceiling): mean {gas.mean():.4f}")
print(f"matched-local  (three-factor): mean {las.mean():.4f}  (floor needs >=0.65)")
print(f"gap_closed: mean {mean_gap:.4f}  LCB {lcb:.4f}  (needs LCB>0.5)")
floor_ok = las.mean()>=0.65
gap_ok = lcb>0.5
print(f"floor {'PASS' if floor_ok else 'FAIL'} | gap {'PASS' if gap_ok else 'FAIL'} -> "
      f"{'PASS' if (floor_ok and gap_ok) else 'FAIL'}")
