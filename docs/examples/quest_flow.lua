-- quest_flow.lua
-- Demonstrates the quest Lua API: accept, progress, complete.
-- Load this script via ScriptCommand::Load or story graph Event node.

-- Accept a quest from the quest board
quest.accept("village_defense")
log("Quest 'village_defense' accepted!")

-- Later, when the player kills an enemy:
quest.progress("village_defense", "kill_bandits", 1)
log("Bandit killed! Progress updated.")

-- When all objectives are done:
quest.complete("village_defense")
log("Quest complete! Return to the village elder.")

-- If the player wants to abandon:
-- quest.abandon("village_defense")
