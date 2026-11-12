import sys,re
p=sys.argv[1]
rows=[];f=False
for ln in open(p):
    if ln.startswith('Sort by top of stack'): f=True; continue
    if f:
        m=re.match(r'\s+(\S+)\s+\(in ([^)]*)\)\s+(\d+)\s*$',ln)
        if m: rows.append((m.group(1),m.group(2),int(m.group(3))))
        elif ln.strip()=='' and rows: break
def short(n):
    m=re.search(r'\d+(shd_attention|shd_matched_arms|shd_matched)(\d+)(\w+?)(?:KB|EB|$)',n)
    return n if len(n)<60 else n[:90]
tot=sum(r[2] for r in rows)
idle=sum(r[2] for r in rows if r[0] in('__psynch_cvwait','swtch_pri'))
busy=tot-idle
print('total',tot,'idle',idle,'busy',busy)
mm=0
for n,l,c in rows[:14]:
    isidle=n in('__psynch_cvwait','swtch_pri')
    print(f'{c:8d} all={100*c/tot:5.1f}%  busy={"" if isidle else format(100*c/busy,"5.1f")+"%"}  {n[:100]}')
def share(pred):
    return sum(c for n,l,c in rows if pred(n))
mat=lambda n: ('dot_columns' in n) or ('transpose' in n) or ('dot_against' in n) or ('axpy' in n)
s=share(mat)
print('matmul-like (dot_columns*, transposes, separately-symbolised axpy): all=%.1f%% busy=%.1f%%'%(100*s/tot,100*s/busy))
print('Amdahl ceiling (infinitely fast, of busy): %.2fx ; of all samples: %.2fx'%(busy/(busy-s), tot/(tot-s)))
dc=share(lambda n:'dot_columns' in n)
print('dot_columns only: busy=%.1f%% ceiling %.2fx'%(100*dc/busy,busy/(busy-dc)))
