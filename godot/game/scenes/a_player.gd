extends Player


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass # Replace with function body.

func _on_cube_spawner_load_complete() -> void:
	self.freeze = false
	pass # Replace with function body.
