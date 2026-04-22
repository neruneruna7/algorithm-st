import random
rand = random.Random()

def load(n): print("\tLOAD %d" % n)
def loadc(n): print("\tLOAD =%d" % n )
def store(n): print("\tSTORE %d" % n)
def storec(n): print("\tSTORE =%d" % n)
def storei(n): print("\tSTORE *%d" % n)
def mult(n): print("\tMULT %d" % n)
def multc(n): print("\tMULT =%d" % n)
def add(n): print("\tADD %d" % n)
def addc(n): print("\tADD =%d" % n)
def halt(): print("\tHALT")

def make_data(n):
    data = []
    for i in range(n):
        data.append(rand.randint(0, 1000))
    return data

def init_data(n):
    data = make_data(n)
    print(";", data)
    loadc(n)
    store(1)
    for i, x in enumerate(range(n)):
        loadc(data[i])
        store(i + 2)

lcount = 0
def mklabel():
    global lcount
    label = "lable%d" % lcount
    lcount += 1
    print(label + ":")
    return label

def block_move(n):
    load(1)
    addc(2) 
    store(1)  # r1 = n ; r1 <- r1 + 2
    load(2)
    storei(1) # [r1] <- r2 {[n + 2] <- r2}
    load(1)
    addc(1)  
    store(1)  # r1 <- r1 + 1 
    load(3)
    storei(1) # [r1] <- r3 {[n + 3] <- r3}
    loadc(4)
    store(3)
    load(3)
    addc(1)
    store(3)
    label1 = mklabel()
    
    
    
    

init_data(10)
block_move(10)
halt()


