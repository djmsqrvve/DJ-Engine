-- npc_dialogue.lua
-- Demonstrates reading custom document data from Lua.
-- Use to look up NPC data, ability definitions, or item stats.

-- Look up an ability definition from custom documents
local fireball_json = ecs.get_document("abilities", "fireball")
if fireball_json then
    log("Fireball data: " .. fireball_json)
else
    log("Fireball ability not found in documents")
end

-- Move an NPC to a new position (e.g., during a cutscene)
-- First find the NPC entity
local entities = ecs.get_entities()
for _, e in ipairs(entities) do
    if e.name == "village_elder" then
        ecs.set_position(e.entity_id, 200.0, 100.0)
        log("Moved village elder to (200, 100)")
        break
    end
end

-- Hide an entity (e.g., make a door disappear after opening)
-- ecs.set_field(door_entity_id, "visibility", "visible", false)
