import sys,re,statistics as st
for p in sys.argv[1:]:
    rows={'r4':[],'var':[]}
    for ln in open(p):
        m=re.match(r'(\d+) (\w+) wall=(\S+) user=(\S+) instructions=(\d+) cycles=(\d+) sha=(\w+)',ln)
        if m: rows[m.group(2)].append(dict(wall=float(m.group(3)),user=float(m.group(4)),ins=int(m.group(5)),cyc=int(m.group(6)),sha=m.group(7)[:12]))
    print(p.split('/')[-1], 'n=',{k:len(v) for k,v in rows.items()})
    for k,v in rows.items():
        if not v: continue
        f=lambda key:(min(r[key] for r in v),st.median(r[key] for r in v))
        print(f' {k}: ins min {f("ins")[0]/1e9:.1f}G med {f("ins")[1]/1e9:.1f}G | cyc min {f("cyc")[0]/1e9:.1f}G med {f("cyc")[1]/1e9:.1f}G | user med {f("user")[1]:.1f}s | wall med {f("wall")[1]:.1f}s | shas {sorted(set(r["sha"] for r in v))}')
    if rows['r4'] and rows['var']:
        a,b=rows['r4'],rows['var']
        for key in('ins','cyc','user','wall'):
            print(f'  speedup {key}: min {min(r[key] for r in a)/min(r[key] for r in b):.3f}x  median {st.median(r[key] for r in a)/st.median(r[key] for r in b):.3f}x')
