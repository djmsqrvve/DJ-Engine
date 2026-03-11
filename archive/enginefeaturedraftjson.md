Here is a single, engine-agnostic JSON-style schema for dj_engine’s editor surface. You can adapt this into real JSON Schema, Rust structs, or Bevy resources.

Top-level schema
text
{
  "project": {
    "id": "string",
    "name": "string",
    "version": "string",
    "settings": { "$ref": "#/definitions/ProjectSettings" },
    "editorPreferences": { "$ref": "#/definitions/EditorPreferences" },
    "scenes": [{ "$ref": "#/definitions/Scene" }],
    "storyGraphs": [{ "$ref": "#/definitions/StoryGraph" }],
    "database": { "$ref": "#/definitions/Database" },
    "assets": { "$ref": "#/definitions/AssetIndex" }
  }
}
Definitions
Project / editor settings
text
{
  "definitions": {
    "ProjectSettings": {
      "platforms": ["pc", "console", "web", "mobile"],
      "defaultResolution": { "width": "int", "height": "int" },
      "targetFps": "int",
      "vsync": "bool",
      "pixelPerfect": "bool",
      "inputProfile": "enum: [jrpg, rts, hybrid]",
      "localization": {
        "languages": ["string"],         // ["en", "fr", "jp"]
        "defaultLanguage": "string"
      },
      "paths": {
        "scenes": "string",
        "storyGraphs": "string",
        "database": "string",
        "assets": "string"
      },
      "autosave": {
        "enabled": "bool",
        "intervalSeconds": "int",
        "maxBackups": "int"
      }
    },

    "EditorPreferences": {
      "theme": "enum: [light, dark]",
      "uiScale": "float",
      "fontSize": "int",
      "gridSize": { "x": "int", "y": "int" },
      "snap": {
        "position": "float",
        "rotation": "float",
        "scale": "float",
        "enabled": "bool"
      },
      "defaultGizmoMode": "enum: [move, rotate, scale]",
      "keybindings": { "string": "string" },   // action -> key
      "layoutPreset": "enum: [jrpg_mapping, td_balancing, custom]"
    }
  }
}
Scene, layers, entities
text
{
  "definitions": {
    "Scene": {
      "id": "string",
      "name": "string",
      "type": "enum: [jrpg, td, shared]",
      "sizeTiles": { "width": "int", "height": "int" },
      "tileSize": { "width": "int", "height": "int" },
      "backgroundColor": "color",
      "defaultSpawn": {
        "player": { "x": "float", "y": "float" },
        "camera": { "x": "float", "y": "float" }
      },
      "audio": {
        "musicTrackId": "string|null",
        "loop": "bool"
      },
      "scripts": {
        "onEnter": "string|null",     // Lua script id
        "onExit": "string|null"
      },
      "layers": [{ "$ref": "#/definitions/Layer" }],
      "entities": [{ "$ref": "#/definitions/Entity" }],
      "pathfinding": {
        "enabled": "bool",
        "grid": {
          "width": "int",
          "height": "int",
          "cells": [
            {
              "x": "int",
              "y": "int",
              "walkable": "bool",
              "buildable": "bool"
            }
          ]
        }
      }
    },

    "Layer": {
      "id": "string",
      "name": "string",
      "order": "int",
      "visible": "bool",
      "locked": "bool",
      "parallax": { "x": "float", "y": "float" }
    },

    "Entity": {
      "id": "string",
      "name": "string",
      "type": "enum: [npc, enemy, tower, trigger, prop, deco, spawner, ui, other]",
      "layerId": "string",
      "parentId": "string|null",
      "prefabId": "string|null",
      "components": {
        "transform": { "$ref": "#/definitions/TransformComponent" },
        "sprite": { "$ref": "#/definitions/SpriteComponent", "optional": true },
        "collision": { "$ref": "#/definitions/CollisionComponent", "optional": true },
        "interactivity": { "$ref": "#/definitions/InteractivityComponent", "optional": true },
        "npc": { "$ref": "#/definitions/NpcComponent", "optional": true },
        "enemy": { "$ref": "#/definitions/EnemyComponent", "optional": true },
        "combatStats": { "$ref": "#/definitions/CombatStatsComponent", "optional": true },
        "tower": { "$ref": "#/definitions/TowerComponent", "optional": true },
        "spawner": { "$ref": "#/definitions/SpawnerComponent", "optional": true },
        "audioSource": { "$ref": "#/definitions/AudioSourceComponent", "optional": true },
        "cameraAnchor": { "$ref": "#/definitions/CameraAnchorComponent", "optional": true },
        "custom": { "string": "any" }   // extensibility
      }
    }
  }
}
Component schemas
text
{
  "definitions": {
    "TransformComponent": {
      "position": { "x": "float", "y": "float", "z": "float" },
      "rotation": { "x": "float", "y": "float", "z": "float" },
      "scale": { "x": "float", "y": "float", "z": "float" },
      "lockUniformScale": "bool"
    },

    "SpriteComponent": {
      "spriteId": "string",
      "sortingLayer": "string",
      "sortingOrder": "int",
      "tint": "color",
      "flipX": "bool",
      "flipY": "bool",
      "animation": {
        "clipId": "string|null",
        "speed": "float",
        "loop": "bool"
      }
    },

    "CollisionComponent": {
      "enabled": "bool",
      "bodyType": "enum: [static, dynamic, kinematic]",
      "shape": "enum: [box, circle, polygon]",
      "box": { "width": "float", "height": "float" },
      "circle": { "radius": "float" },
      "polygon": { "points": [{ "x": "float", "y": "float" }] },
      "offset": { "x": "float", "y": "float" },
      "layer": "string",
      "mask": ["string"],
      "isTrigger": "bool"
    },

    "InteractivityComponent": {
      "triggerType": "enum: [none, door, chest, npc, custom]",
      "triggerId": "string",
      "parameters": { "string": "any" },
      "luaScriptId": "string|null",
      "events": {
        "onInteract": "string|null",
        "onEnter": "string|null",
        "onExit": "string|null",
        "onDeath": "string|null"
      }
    },

    "NpcComponent": {
      "npcId": "string",            // link to DB
      "displayName": { "string": "string" }, // per language
      "dialogueSetId": "string",
      "questIds": ["string"],
      "inventoryPresetId": "string|null",
      "faction": "string"
    },

    "EnemyComponent": {
      "enemyId": "string",          // link to DB
      "behaviorProfileId": "string",
      "aggroRange": "float",
      "patrolPathId": "string|null"
    },

    "CombatStatsComponent": {
      "maxHp": "int",
      "hp": "int",
      "mana": "int",
      "damage": "int",
      "defense": "int",
      "attackSpeed": "float",
      "moveSpeed": "float",
      "critChance": "float",
      "lootTableId": "string"
    },

    "TowerComponent": {
      "towerId": "string",          // link to DB
      "damage": "int",
      "range": "float",
      "cooldown": "float",
      "buildCost": "int",
      "buildTime": "float",
      "upgradePathId": "string|null",
      "targetingMode": "enum: [first, last, closest, strongest]",
      "projectileId": "string",
      "effectId": "string|null"
    },

    "SpawnerComponent": {
      "waveCount": "int",
      "spawnInterval": "float",
      "startDelay": "float",
      "loop": "bool",
      "waves": [{
        "enemyTemplateId": "string",
        "count": "int",
        "interval": "float"
      }],
      "pathId": "string|null"
    },

    "AudioSourceComponent": {
      "clipId": "string",
      "volume": "float",
      "loop": "bool",
      "spatial": "bool"
    },

    "CameraAnchorComponent": {
      "bounds": {
        "minX": "float",
        "maxX": "float",
        "minY": "float",
        "maxY": "float"
      },
      "followEntityId": "string|null"
    }
  }
}
Story graph schema
text
{
  "definitions": {
    "StoryGraph": {
      "id": "string",
      "name": "string",
      "description": "string",
      "type": "enum: [dialogue, cutscene, missionLogic]",
      "rootNodeId": "string",
      "variables": { "string": "any" },       // optional initial values
      "nodes": [{ "$ref": "#/definitions/StoryNode" }]
    },

    "StoryNode": {
      "id": "string",
      "type": "enum: [dialogue, choice, action, conditional, camera, timeControl, end]",
      "position": { "x": "float", "y": "float" },   // editor canvas
      "data": "union: DialogueNodeData | ChoiceNodeData | ActionNodeData | ConditionalNodeData | CameraNodeData | TimeControlNodeData | EndNodeData"
    },

    "DialogueNodeData": {
      "speakerId": "string",
      "portraitId": "string|null",
      "text": { "string": "string" },     // per language
      "voiceLineId": "string|null",
      "duration": "float|null",
      "nextNodeId": "string|null"
    },

    "ChoiceNodeData": {
      "prompt": { "string": "string" },
      "options": [{
        "id": "string",
        "text": { "string": "string" },
        "targetNodeId": "string",
        "conditions": [{ "$ref": "#/definitions/StoryCondition" }],
        "effects": [{ "$ref": "#/definitions/StoryEffect" }]
      }]
    },

    "ActionNodeData": {
      "luaScriptId": "string",
      "params": { "string": "any" },
      "nextNodeId": "string|null"
    },

    "ConditionalNodeData": {
      "condition": { "$ref": "#/definitions/StoryCondition" },
      "trueTargetNodeId": "string",
      "falseTargetNodeId": "string"
    },

    "CameraNodeData": {
      "presetId": "string|null",
      "position": { "x": "float", "y": "float", "z": "float" },
      "zoom": "float",
      "angle": "float",
      "duration": "float",
      "easing": "string",
      "nextNodeId": "string|null"
    },

    "TimeControlNodeData": {
      "pauseGameplay": "bool",
      "timeScale": "float",
      "nextNodeId": "string|null"
    },

    "EndNodeData": {
      "endType": "enum: [returnToGameplay, loadScene, quit]",
      "targetSceneId": "string|null"
    },

    "StoryCondition": {
      "variable": "string",
      "operator": "enum: [==, !=, <, <=, >, >=, contains]",
      "value": "any"
    },

    "StoryEffect": {
      "type": "enum: [setVar, addVar, giveItem, removeItem, setQuestState]",
      "params": { "string": "any" }
    }
  }
}
Database schema (items, NPCs, towers, etc.)
text
{
  "definitions": {
    "Database": {
      "items": [{ "$ref": "#/definitions/ItemRow" }],
      "npcs": [{ "$ref": "#/definitions/NpcRow" }],
      "towers": [{ "$ref": "#/definitions/TowerRow" }],
      "enemies": [{ "$ref": "#/definitions/EnemyRow" }],
      "lootTables": [{ "$ref": "#/definitions/LootTableRow" }],
      "quests": [{ "$ref": "#/definitions/QuestRow" }]
    },

    "ItemRow": {
      "id": "string",
      "name": { "string": "string" },
      "type": "enum: [weapon, armor, potion, currency, questItem, misc]",
      "damage": "int",
      "defense": "int",
      "healAmount": "int",
      "price": "int",
      "sellValue": "int",
      "maxStack": "int",
      "rarity": "enum: [common, uncommon, rare, epic, legendary]",
      "spriteId": "string",
      "description": { "string": "string" },
      "scripts": {
        "onUse": "string|null",
        "onEquip": "string|null",
        "onUnequip": "string|null"
      }
    },

    "NpcRow": {
      "id": "string",
      "name": { "string": "string" },
      "dialogueSetId": "string",
      "locationTags": ["string"],
      "defaultFaction": "string",
      "defaultQuestIds": ["string"],
      "lootTableId": "string|null",
      "portraitId": "string"
    },

    "TowerRow": {
      "id": "string",
      "name": { "string": "string" },
      "damage": "int",
      "range": "float",
      "cooldown": "float",
      "cost": "int",
      "buildTime": "float",
      "upgradeToId": "string|null",
      "projectileId": "string",
      "effectId": "string|null",
      "description": { "string": "string" }
    },

    "EnemyRow": {
      "id": "string",
      "name": { "string": "string" },
      "hp": "int",
      "damage": "int",
      "speed": "float",
      "experience": "int",
      "lootTableId": "string",
      "behaviorProfileId": "string"
    },

    "LootTableRow": {
      "id": "string",
      "entries": [{
        "itemId": "string",
        "chance": "float",
        "minQuantity": "int",
        "maxQuantity": "int"
      }]
    },

    "QuestRow": {
      "id": "string",
      "name": { "string": "string" },
      "description": { "string": "string" },
      "startConditions": [{ "string": "any" }],
      "completionConditions": [{ "string": "any" }],
      "rewards": {
        "gold": "int",
        "experience": "int",
        "itemRewards": [{
          "itemId": "string",
          "quantity": "int"
        }],
        "flags": { "string": "any" }
      }
    }
  }
}
Assets, prefabs, validation hooks
text
{
  "definitions": {
    "AssetIndex": {
      "sprites": [{ "id": "string", "path": "string", "tags": ["string"] }],
      "audio": [{ "id": "string", "path": "string", "type": "enum: [music, sfx, voice]" }],
      "scripts": [{ "id": "string", "path": "string" }],
      "prefabs": [{ "$ref": "#/definitions/Prefab" }],
      "storyGraphs": [{ "id": "string", "path": "string" }],
      "scenes": [{ "id": "string", "path": "string" }]
    },

    "Prefab": {
      "id": "string",
      "name": "string",
      "entity": { "$ref": "#/definitions/Entity" }
    }
  }
}