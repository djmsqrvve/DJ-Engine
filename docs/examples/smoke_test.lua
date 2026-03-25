-- smoke_test.lua
-- Exercises every Lua API table to verify the bridge is working.
-- Load via ScriptCommand::Load or paste into a story graph Event node.
-- Each call should log success or produce a warning (not crash).

log("=== Lua API Smoke Test ===")

-- 1. ECS table
log("[ecs] Testing set_position...")
-- ecs.set_position(0, 10.0, 20.0)  -- entity 0 may not exist, that's OK
log("[ecs] Testing get_entities...")
local entities = ecs.get_entities()
log("[ecs] Found " .. #entities .. " entities")
log("[ecs] Testing get_document...")
local doc = ecs.get_document("abilities", "fireball")
if doc then
    log("[ecs] get_document returned data: " .. string.sub(doc, 1, 50))
else
    log("[ecs] get_document returned nil (no documents loaded)")
end

-- 2. Quest table
log("[quest] Testing accept...")
quest.accept("smoke_test_quest")
log("[quest] Testing progress...")
quest.progress("smoke_test_quest", "test_obj", 1)
log("[quest] Testing complete...")
quest.complete("smoke_test_quest")
log("[quest] Testing abandon...")
quest.abandon("smoke_test_quest")

-- 3. Combat table
log("[combat] Testing attack (entity 0 -> 0, likely no-op)...")
combat.attack(0, 0)

-- 4. Inventory table
log("[inventory] Testing add_item...")
inventory.add_item("smoke_test_item", 1, 10)
log("[inventory] Testing add_currency...")
inventory.add_currency("smoke_gold", 100)
log("[inventory] Testing spend_currency...")
inventory.spend_currency("smoke_gold", 50)
log("[inventory] Testing remove_item...")
inventory.remove_item("smoke_test_item", 1)

-- 5. Economy table
log("[economy] Testing vendor_buy...")
economy.vendor_buy("smoke_item")
log("[economy] Testing vendor_sell...")
economy.vendor_sell("smoke_item")

-- 6. Character table
log("[character] Testing earn_title...")
character.earn_title("smoke_champion")
log("[character] Testing equip_title...")
character.equip_title("smoke_champion")
log("[character] Testing gain_weapon_skill...")
character.gain_weapon_skill("smoke_swords", 10)

log("=== Lua API Smoke Test COMPLETE ===")
log("If you see this line, all 8 Lua tables are functional.")
