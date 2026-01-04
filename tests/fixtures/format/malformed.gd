########################################
#GODOT 3.5.1v
#Add this script in your AnimationPlayer
########################################

#We need to get the functionality of our animationplayer method (ex: play and stop)
extends AnimationPlayer

#We add a simple function inside _ready virtual method 
func _ready() -> void:

#start animation:
	self.play("Prop-Main-Spin-loop")
  
#stop animation (if you need it or not):
#	self.stop()
  
########################################
