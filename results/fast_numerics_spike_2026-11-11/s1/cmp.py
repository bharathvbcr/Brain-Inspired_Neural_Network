import json,glob,sys
ex=json.load(open(sys.argv[1]))
for p in sys.argv[2:]:
    v=json.load(open(p))
    diff=[k for k in ex if k!='wall_secs' and ex[k]!=v.get(k)]
    print(p.split('/')[-2], 'differing fields:',diff)
    for k in diff:
        if k in('accuracy','mean_loss','mean_gradient_norm','mean_update_rms','mean_firing_rate'): print('   ',k,ex[k],v[k])
