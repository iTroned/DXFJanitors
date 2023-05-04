When merging two or more layers we can get duplicate values. Example: (2.0, 2.0) can be on line A from layer1 and line B from layer2.
When combining the layers we will get duplicate values for the same point. The algorithm gets confused and is bugged when trying to use close or extend function.

Connect and extend functions: Affects unchecked layers
