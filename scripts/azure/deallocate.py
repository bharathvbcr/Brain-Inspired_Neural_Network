#!/usr/bin/env python3
"""Deallocate the campaign scale set without deleting results or infrastructure."""

import subprocess

from scripts.azure.launch import RG, VMSS

subprocess.run(["az", "vmss", "deallocate", "--resource-group", RG, "--name", VMSS], check=True)
print(f"deallocated {VMSS}; storage and results were retained")
