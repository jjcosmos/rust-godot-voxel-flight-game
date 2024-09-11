extends Label



func _on_cube_spawner_current_chunk_changed(chunk_idex:Vector3i) -> void:
	text = str(chunk_idex)
