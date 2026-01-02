extends Node

signal my_signal
signal BadSignal

const MAX_VALUE = 100
const badConstant = 50

var my_variable = 0
var BadVariable = 1

func my_function():
    pass

func BadFunction():
    var x = 1
    pass

func _ready():
    var unused_param = 0
    print("ready")

func compare_test():
    var x = 5
    if x == x:
        print("always true")

class InnerClass:
    var inner_var = 0

    func inner_method():
        pass
