-- inventory_rewards.lua
-- Demonstrates giving rewards via the Lua inventory API.
-- Called from quest turn-in or treasure chest scripts.

-- Give the player gold
inventory.add_currency("gold", 100)
log("Received 100 gold!")

-- Give items
inventory.add_item("health_potion", 3, 10) -- 3 potions, max stack 10
inventory.add_item("iron_sword", 1, 1)     -- 1 sword, max stack 1
log("Received 3 health potions and an iron sword!")

-- Spend currency (returns silently if insufficient)
inventory.spend_currency("gold", 25)
log("Spent 25 gold at the shop")

-- Remove items (for quest turn-in)
inventory.remove_item("wolf_pelt", 5)
log("Turned in 5 wolf pelts")
