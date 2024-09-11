extends GPUParticles3D

func _ready() -> void:
	self.finished.connect(on_finish)
	emitting = true


func on_finish() -> void:
	get_parent().queue_free()