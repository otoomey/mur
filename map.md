Desiderata:
1. Lazy evaluation
2. stateful vs stateless processing
3. Sub-streams
4. The PE should be fed one item per cycle per SIMD lane (if there are SIMD lanes), i.e. maximum pipeline utilisation

stream:
- reg: hardware PE register associated with this stream
- addr: address of the thing in memory
- len: number of items
- size: size of each item

## Why
We expect to see a speed up in 3 areas:
1. PE closer to memory means potentially lower latency
2. Separating memory operations from execution increases throughput
3. Letting the cpu execute something else in the mean time is good

## Ideas:
1. The instructions that construct the stream process the first minimum set of items in that stream
2. When the stream is consumed, it actually executes

Advantages:
- lazy evaluation doable
- substreams possible, limited by stack (stack is interesting)
- potentially stateless if hart has to repeat commands for each group

Disadvantages:
- hart potentially has to repeat commands (if we want stateless)

If multiple harts can issue commands, then fundamentally some things need to happen:
- Each in-memory PE must know what to do with the data when it's done (i.e. where to put it in memory; as it MUST go back to memory)
- OR harts have to take turns accessing the PEs. If the PE is not done (how does it know when it's done?), it refuses to accept another core

Blocking is preferable - shitty code should not be optimised for. But this means the PE must have a way of knowing it has finished...

The hart issues commands to the PE. These commands instruct the execution and address generation pipelines what to do and when. 

Communication to PE:
- if something else is executing there, then the instruction blocks
- Otherwise you can start queuing. The PE immediately begins execution.

Communicating back to Hart:
- the consume command sent last to the Hart determines which soft interrupt to fire and an optional return value (if there is no consume command then the map operation is equivalent to a NOP and we issue an exception or general interrupt instead. Nice)
- the hart can decide when it wants to consume the interrupt, if ever. This should be easy enough to statically compile, just consume the interrupt before the programmer accesses the corresponding pointer or return register or whatever. If they do pointer shenanigans it's their own fault, we force the compiler to consume the interrupt immediately to ensure sanity.

