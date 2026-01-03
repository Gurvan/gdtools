extends Node

var gltf := GLTFDocument.new()
var gltf_state := GLTFState.new()
var gltf_model_state: GLTFState = null

# This is used to change the keys in a file to a new key
var key_remap := {
	"air_movement/land": "collision/ground/land",
	"air_movement/special_land": "collision/ground/special_land",
	"downed/facing_up/collision": "collision/ground/face_up_bounce",
	"downed/facing_down/collision": "collision/ground/face_down_bounce",
	"defense/tech_ground": "collision/ground/tech",
	"defense/tech_ground_forward": "collision/ground/tech_forward",
	"defense/tech_ground_backward": "collision/ground/tech_backward",
	"defense/tech_wall": "collision/wall/tech",
	"defense/tech_wall_jump": "collision/wall/tech_jump",
	"defense/tech_ceil": "collision/ceil/tech",
}


# Called when the node enters the scene tree for the first time.
func _ready():
	GLTFDocument.register_gltf_document_extension(RuckusKeepIntactExtension.new())
	var node := Node3D.new()
	var err := gltf.append_from_scene(node, gltf_state)
	if err != OK:
		print("Append Error: " + str(err) + " " + error_string(err))


func get_animation_list() -> Array[String]:
	return gltf_state.get_unique_animation_names()


func get_bone_list() -> Array:
	# TODO: Should we cache this or something? Maybe just load it when file is loaded
	var result = []
	var skeleton = %Preview3D.skeleton
	for bone_idx in range(skeleton.get_bone_count()):
		var bone_name = skeleton.get_bone_name(bone_idx)
		result.append(bone_name)
	return result


func load_file(path: String):
	# Reset gltf variables
	gltf_state = GLTFState.new()
	gltf.append_from_file(path, gltf_state)

	var extras = gltf_state.json.get("extras", {})

	# Suppress signals during bulk loading to avoid multiple UI rebuilds
	Fighter.begin_bulk_update()

	# Load actions, attributes, and entities. We will also automatically load
	# these from the sidecar files if they aren't set in the glb. This helps
	# in the case where we overwrite the glb from Blender, it saves us having
	# to manually re-import everything again. Could make it optional if it
	# causes issues
	var actions = extras.get("actions", [])
	_load_actions(actions)
	var actions_path = path.get_basename() + "_actions.json"
	if actions.is_empty() && FileAccess.file_exists(actions_path):
		import_actions(actions_path)  # TODO: Should we auto-save if this happens?

	var attributes = extras.get("attributes", {})
	_load_attributes(attributes)
	var attributes_path = path.get_basename() + "_attributes.json"
	if attributes.is_empty() && FileAccess.file_exists(attributes_path):
		import_attributes(attributes_path)  # TODO: Should we auto-save if this happens?

	var entities = extras.get("entities", {})
	_load_entities(entities)
	var entities_path = path.get_basename() + "_entities.json"
	if entities.is_empty() && FileAccess.file_exists(entities_path):
		import_entities(entities_path)  # TODO: Should we auto-save if this happens?

	# End bulk update - this emits all change signals once
	Fighter.end_bulk_update()

	%Preview3D.set_gltf(gltf, gltf_state)
	%Toolbar/ModelTool.disabled = false
	%Toolbar/FileTool.get_popup().set_item_disabled(1, false)


	# TODO: Load / cache bone list here?
func save_file(path: String, _with_model: bool = false):
	var actions = _convert_actions_to_arr()
	var attributes = _convert_attributes()
	var entities = _convert_entities()

	print("Saving gltf file at %s" % path)

	# Since we're adding thing to the state, better make a fresh copy in case
	# of successive saves.
	var gltf_state_out = gltf_state.duplicate(true)
	# additional data seems to not be duplicated, doing it manually
	gltf_state_out.set_additional_data("ruckus_keep_intact", gltf_state.get_additional_data("ruckus_keep_intact"))

	# This doesn't work at the moment. When Godot loads a gltf file that contains a model,
	# it seems to add the model data multiple times, so the saved file grows at each load/save,
	# so we just disable saving files with models altogether.
	# if with_model and gltf_model_state != null and len(gltf_state_out.skins) > 0:
	# 	# Find the node containing the mesh (3D model).
	# 	var nodes := gltf_model_state.nodes
	# 	for node in nodes:
	# 		if node.mesh == -1:
	# 			continue
	# 		if node.resource_name == "ModelMesh":
	# 			# Ensure that the indices of the joints match.
	# 			gltf_model_state.skins[0].joints = gltf_state_out.skins[0].joints
	# 			gltf_state_out.set_skins(gltf_model_state.skins)
	# 			gltf_state_out.set_meshes(gltf_model_state.meshes)
	# 		gltf_state_out.set_nodes(gltf_state_out.nodes + [node])
	# Keep existing extras
	var extras = gltf_state.json.get("extras", {})
	extras["actions"] = actions
	extras["attributes"] = attributes
	extras["entities"] = entities
	gltf_state_out.json["extras"] = extras

	var err = gltf.write_to_filesystem(gltf_state_out, path)
	if err != OK:
		print("Save Error: " + str(err) + " " + error_string(err))

	# Also save actions and attributes files (easier to see what
	# changed in git)
	var actions_path = path.get_basename() + "_actions.json"
	export_actions(actions_path)
	var attributes_path = path.get_basename() + "_attributes.json"
	export_attributes(attributes_path)
	var entities_path = path.get_basename() + "_entities.json"
	export_entities(entities_path)


func _export_as_json(data, path: String):
	JsonUtils.clean_float_values(data)
	# We deliberately set the full_precision (4th argument) to false here
	# to prevent values like 1.1 from being converted to 1.10000000000000009
	var json_string = JSON.stringify(data, "\t", true, false)
	var file = FileAccess.open(path, FileAccess.WRITE)
	file.store_string(json_string)


func _import_from_json(path: String):
	var str_contents = FileAccess.get_file_as_string(path)
	var contents = JSON.parse_string(str_contents)
	if !contents:
		# TODO: Maybe return / display error
		return
	# TODO: Handle other errors like the json contents not being correct
	return contents


func export_actions(path: String):
	var actions = _convert_actions_to_arr()
	_export_as_json(actions, path)


func import_actions(path: String):
	var contents = _import_from_json(path)
	if !contents:
		return
	_load_actions(contents)


func export_attributes(path: String):
	var attributes = _convert_attributes()
	_export_as_json(attributes, path)


func import_attributes(path: String):
	var contents = _import_from_json(path)
	if !contents:
		return
	_load_attributes(contents)


func export_entities(path: String):
	var entities = _convert_entities()
	_export_as_json(entities, path)


func import_entities(path: String):
	var contents = _import_from_json(path)
	if !contents:
		return
	_load_entities(contents)


func load_model_file(path: String):
	gltf_model_state = GLTFState.new()
	gltf.append_from_file(path, gltf_model_state)

	# Find the node containing the mesh (3D model).
	var nodes := gltf_model_state.nodes
	for node in nodes:
		if node.mesh == -1:
			continue
		# Tries to rename nodes to common names
		if node.resource_name == "main":
			node.resource_name = "ModelMesh"
		elif !node.resource_name.begins_with("ModelMesh_"):
			node.resource_name = "ModelMesh_" + str(node.resource_name)

	%Preview3D.set_model(gltf, gltf_model_state)


func _load_actions(actions: Array):
	# Reset fighter actions
	Fighter.reset_actions()

	# Load script and triggers
	for action in actions:
		var key = action.key
		var script = action.script

		#if key_remap.has(key):
			#print(key, " => ", key_remap[key])
		# Create action in Fighter
		var remapped_key = key_remap.get(key, key)
		var fa = Fighter.add_action(remapped_key)
		_parse_script(fa, script)
		fa.properties = Serializable.from_dict(action.properties)


	# UI reload handled via Fighter.actions_changed signal
func _load_attributes(attributes: Dictionary):
	# Reset fighter attributes
	Fighter.reset_attributes()

	var common = attributes.get("common")
	if common != null:
		Fighter.attributes = Serializable.from_dict(common)

	var hurtboxes = attributes.get("hurtboxes")
	if hurtboxes != null:
		Fighter.hurtboxes = []
		for i in range(len(hurtboxes)):
			var hb = hurtboxes[i]
			Fighter.hurtboxes.push_back(Serializable.from_dict(hb))

	var bones = attributes.get("bones")
	if bones != null:
		Fighter.bones = []
		for i in range(len(bones)):
			var b = bones[i]
			Fighter.bones.push_back(Serializable.from_dict(b))

	var lgb = attributes.get("ledge_grab_box")
	if lgb != null:
		Fighter.lgb = Serializable.from_dict(lgb)

	var ecb = attributes.get("environment_collision_box")
	if ecb != null:
		Fighter.ecb = Serializable.from_dict(ecb)

	var nudge_box = attributes.get("nudge_box")
	if nudge_box != null:
		Fighter.nudge_box = Serializable.from_dict(nudge_box)


	# UI reload handled via Fighter.attributes_changed signal
func _convert_actions_to_arr() -> Array[Dictionary]:
	var actions: Array[Dictionary] = []
	for key in Fighter.actions:
		var fa = Fighter.actions[key]

		# Serialize trigger objects
		var script = {}
		for frame_num in fa.frame_triggers:
			var fts = fa.frame_triggers[frame_num]
			var serialized_fts = []
			for ft in fts:
				if ft == null:
					continue
				serialized_fts.push_back(ft.to_dict())
			script[frame_num] = serialized_fts

		# Add frame triggers to extras dict
		actions.push_back({
			"key": key,
			"script": script,
			"properties": fa.properties.to_dict(),
		})
	return actions


func _convert_attributes() -> Dictionary:
	var _array_to_dicts = func(arr):
		var dicts = []
		for i in range(len(arr)):
			dicts.push_back(arr[i].to_dict())
		return dicts

	var common = Fighter.attributes.to_dict()
	var hurtboxes = _array_to_dicts.call(Fighter.hurtboxes)
	var bones = _array_to_dicts.call(Fighter.bones)
	var lgb = Fighter.lgb.to_dict()
	var ecb = Fighter.ecb.to_dict()
	var nudge_box = Fighter.nudge_box.to_dict()
	var attributes = {
		"common": common,
		"hurtboxes": hurtboxes,
		"bones": bones,
		"ledge_grab_box": lgb,
		"environment_collision_box": ecb,
		"nudge_box": nudge_box,
	}
	return attributes


func _load_entities(entities: Dictionary):
	# Reset fighter entities
	Fighter.reset_entities()

	for entity_name in entities:
		var entity = entities[entity_name]
		var new_entity = EntityState.new()

		var attributes = entity.get("attributes")
		if attributes != null:
			new_entity.attributes = Serializable.from_dict(attributes)

		var ecb = entity.get("environment_collision_box")
		if ecb != null:
			new_entity.ecb = Serializable.from_dict(ecb)

		var scripts = entity.get("scripts")
		if scripts != null:
			for i in range(len(scripts)):
				var script = scripts[i]
				var fa = FighterAction.new(NullProperties.new())
				_parse_script(fa, script)
				new_entity.scripts[i] = fa

		Fighter.entities[entity_name] = new_entity


	# UI reload handled via Fighter.entities_changed signal
func _convert_entities() -> Dictionary:
	var entities: Dictionary = {}
	for key in Fighter.entities:
		#print(key)
		var entity = Fighter.entities[key]
		var attributes = entity.attributes.to_dict()
		var ecb = entity.ecb.to_dict()
		var scripts = []
		for i in entity.scripts:
			#print("\t", i)
			# Serialize trigger objects
			var script = {}
			for frame_num in entity.scripts[i].frame_triggers:
				var fts = entity.scripts[i].frame_triggers[frame_num]
				var serialized_fts = []
				for ft in fts:
					if ft == null:
						continue
					serialized_fts.push_back(ft.to_dict())
				script[frame_num] = serialized_fts
			scripts.push_back(script)
		entities[key] = {
			"attributes": attributes,
			"environment_collision_box": ecb,
			"scripts": scripts,
		}
	return entities


func _parse_script(fa: FighterAction, script: Dictionary):
	fa.frame_triggers = {}  # Reset frame triggers

	for frame_num in script:
		var triggers = []
		for rt in script[frame_num]:
			var dst = Serializable.from_dict(rt)
			# This is here because a field was renamed
			if rt["_type"] == "res://triggers/create_hitbox.gd" && rt.has("element"):
				dst.property = rt["element"]
			triggers.push_back(dst)
		fa.frame_triggers[int(frame_num)] = triggers
