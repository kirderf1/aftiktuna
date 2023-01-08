package me.kirderf.aftiktuna.location;

import me.kirderf.aftiktuna.object.door.DoorType;
import me.kirderf.aftiktuna.object.entity.AftikNPC;
import me.kirderf.aftiktuna.object.entity.Creature;
import me.kirderf.aftiktuna.object.entity.Shopkeeper;
import me.kirderf.aftiktuna.object.entity.Stats;
import me.kirderf.aftiktuna.object.type.CreatureType;
import me.kirderf.aftiktuna.object.type.ItemType;

public interface SymbolType {
	
	void handle(Position pos, LocationBuilder builder);
	
	record Door(String pairId, DoorType type) implements SymbolType {
		@Override
		public void handle(Position pos, LocationBuilder builder) {
			builder.addToDoorPair(pairId, pos, type);
		}
	}
	
	record Recruitable(String name, Stats stats) implements SymbolType {
		@Override
		public void handle(Position pos, LocationBuilder builder) {
			pos.area().addObject(new AftikNPC(name, stats), pos.coord());
		}
	}
	
	record Shop(ItemType... items) implements SymbolType {
		@Override
		public void handle(Position pos, LocationBuilder builder) {
			pos.area().addObject(new Shopkeeper(items), pos.coord());
		}
	}
	
	record ImmovableCreature(CreatureType type) implements SymbolType {
		@Override
		public void handle(Position pos, LocationBuilder builder) {
			pos.area().addObject(new Creature(type, false), pos.coord());
		}
	}
}
