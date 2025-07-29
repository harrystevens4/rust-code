
# About

Library for situations where you want user input while outputting from different threads
```
info
more info <- calling tio.println() causes lines to appear above the prompt
>>> <- prompt
```
```
info
more info
newline
>>>my-stu <- editing is not interupted by lines being printed above
```
Provided minimal line editing, and optional signal and history handling, which should be turned on in the builder. For examples, see lwrap.

Please don't read the source code as it is a total mess. There are a bunch of unnecessary `refcells` and 3 monolithic functions. In my defence, it is pulled from `vanillachat` which was my second ever rust project. (I should not have dived in at the deep end)
