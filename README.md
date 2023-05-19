# 6.5840(6.824) MapReduce but rust version
## challenge
- [ ] Implement your own MapReduce application (see examples in mrapps/*), e.g., Distributed Grep (Section 2.3 of the MapReduce paper).


- [ ] Get your MapReduce coordinator and workers to run on separate machines, as they would in practice. You will need to set up your RPCs to communicate over TCP/IP instead of Unix sockets (see the commented out line in Coordinator.server()), and read/write files using a shared file system. For example, you can ssh into multiple Athena cluster machines at MIT, which use AFS to share files; or you could rent a couple AWS instances and use S3 for storage.


## 
no test for multiple map


## 
no test for multiple reduce


## early_exit
should use tokio::time::sleep instead of std::time::sleep 
but ........ 😔




# Acknowledgement
[BugenZhao/6.824-MapReduce]()

