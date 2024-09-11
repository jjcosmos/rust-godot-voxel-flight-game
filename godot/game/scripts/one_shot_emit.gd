extends GPUParticles3D

func _ready() -> void:
	self.finished.connect(on_finish)
	emitting = true


func on_finish() -> void:
	queue_free()