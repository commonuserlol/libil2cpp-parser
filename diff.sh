#!/bin/bash

# Credits to https://github.com/nneonneo/Il2CppVersions

vers=($(ls */**/struct.h | sed s/a/.a./g | sed s/b/.b./g | sed s/f/.f./g | sed s/p/.p./g | sort -t. -k1,1n -k2,2n -k3,3n -k4,4d -k5,5n | sed s/.a./a/g | sed s/.b./b/g | sed s/.f./f/g | sed s/.p./p/g))
rm -f struct.diff
for ((i=0; i<${#vers[@]}-1; i++)); do
    diff -purwB --label=${vers[i]} --label=${vers[i+1]} ${vers[i]} ${vers[i+1]} >> struct.diff
done

vers=($(ls */**/api.h | sed s/a/.a./g | sed s/b/.b./g | sed s/f/.f./g | sed s/p/.p./g | sort -t. -k1,1n -k2,2n -k3,3n -k4,4d -k5,5n | sed s/.a./a/g | sed s/.b./b/g | sed s/.f./f/g | sed s/.p./p/g))
rm -f api.diff
for ((i=0; i<${#vers[@]}-1; i++)); do
    diff -purwB --label=${vers[i]} --label=${vers[i+1]} ${vers[i]} ${vers[i+1]} >> api.diff
done
