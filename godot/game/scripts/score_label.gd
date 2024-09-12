extends Label

@export var animation_player: AnimationPlayer;
@export var score_speed: float = 5.0;

var lag_score: float = 0;
var target_score: int = 0;

func _process(delta: float) -> void:
	if (lag_score < target_score):
		lag_score += delta * max(0.5, (target_score - lag_score)) * score_speed;
		text = str(int(lag_score));

		if (!animation_player.is_playing()):
			animation_player.play("text_pulse");
	else:
		animation_player.stop();

func _on_a_player_score_updated(new_score:int) -> void:
	## Happens on reset
	if (new_score < target_score):
		lag_score = new_score;
		text = str(new_score);

	target_score = new_score;