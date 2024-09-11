extends ProgressBar


func _on_cube_spawner_load_progress_updated(frac:float) -> void:
	self.value=frac * 100

func _on_cube_spawner_load_complete() -> void:
	self.get_parent_control().visible = false
