extends Node2D
class_name MyClass

signal my_signal
signal data_received(data, sender)

const MAX_VALUE = 100
const PI: float = 3.14

var health: int = 100
var name: String = "Player"

@export var speed: float = 10.0
@onready var sprite = $Sprite2D

enum State { IDLE, WALKING, RUNNING }

func _ready():
	print("Ready!")
	pass

func calculate(a: int, b: int) -> int:
	return a + b

func process_data(data):
	if data == null:
		return
	elif data.is_empty():
		print("Empty")
	else:
		for item in data:
			print(item)

func example_match(value):
	match value:
		1:
			print("One")
		2:
			print("Two")
		_:
			print("Other")
