#| meta: label

def somefunc(a):
    #| solution <<
    # Comments
    print(a)
    return 2*a
    #| placeholder
    # ...
    # more code
    #| >>


assert somefunc(5) == 10
assert somefunc(10) == 20
