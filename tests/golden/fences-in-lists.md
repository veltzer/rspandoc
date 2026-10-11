# Fences after list items

A fence in an item's first chunk ends the list:

- a
```
x
```

1. a
2. b
   continued
```
y
```

After a blank line or a nested list it joins the item:

- a

  b
```
z
```

- a
  - nested
```
w
```
