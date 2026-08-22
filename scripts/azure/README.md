# Azure CPU campaign

This runner executes the preregistered `azure-d32l4-scope-v1` campaign on four
dedicated `Standard_D64als_v7` workers: 256 vCPUs total. It uses one pinned
`x86-64-v4` binary, a resource-aware 4/8-thread scheduler with at most 16 cells
per node, private Blob Storage, no public IPs, and a 19-hour scale-set
deallocation watchdog. The estimated campaign wall time is 17.83 hours; the
compute watchdog ceiling is about $195.55 at the registered retail price.
The private worker subnet uses one Standard NAT Gateway and one Standard static
IPv4 address for outbound package/bootstrap traffic. This creates no inbound
path to a worker; the fixed 19-hour network ceiling is about $0.95 plus data
processing.

The default command is preparation-only and cannot create an identity or VM:

```bash
python3 scripts/azure/launch.py
```

After explicitly approving the two scoped role assignments, launch once:

```bash
python3 scripts/azure/launch.py --enable-outbound-nat --launch
```

Read progress or stop compute billing without deleting evidence:

```bash
python3 scripts/azure/watch.py
python3 scripts/azure/deallocate.py
```

Download the immutable inputs, all outcomes, gates, logs, and the preregistered
verdict report:

```bash
python3 scripts/azure/collect.py
```

`collect.py` exits 2 until all 252 cells and all four gate reports are present.
An Azure Cost Management budget is notification-only; the VM/node watchdogs and
manual deallocation command are the billing stop mechanisms.

Security impact: launch creates one user-assigned managed identity. It receives
`Storage Blob Data Contributor` only on this campaign's private container and
`Virtual Machine Contributor` only on its own VM scale set, solely so workers
can persist results and deallocate compute. No inbound network rule, public IP,
SAS, account key, or SSH private key is placed on a worker.
