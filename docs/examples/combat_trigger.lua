-- combat_trigger.lua
-- Demonstrates triggering combat from Lua.
-- Use when an NPC script or trap needs to deal damage.

-- Get all entities in the scene
local entities = ecs.get_entities()
log("Found " .. #entities .. " entities")

-- Find the first entity named "player" and "enemy"
local player_id = nil
local enemy_id = nil
for _, e in ipairs(entities) do
    if e.name == "player" then player_id = e.entity_id end
    if e.name == "enemy" then enemy_id = e.entity_id end
end

-- Trigger a normal attack (uses attacker's CombatStatsComponent)
if player_id and enemy_id then
    combat.attack(player_id, enemy_id)
    log("Player attacks enemy!")
end

-- Or deal flat damage (bypasses stats, useful for traps/scripts)
if enemy_id then
    combat.attack(0, enemy_id, 50) -- 50 flat damage from "the environment"
    log("Trap deals 50 damage!")
end
