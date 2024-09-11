extends Node2D

@export var demo_obj: Demo


func _ready() -> void:
	demo_obj.speed_increased.connect(on_speed_increased)
	pass


func _process(delta: float) -> void:
	demo_obj.increase_speed(delta * 10.0)
	pass

func on_speed_increased() -> void:
	print("Speed increased GD!")
