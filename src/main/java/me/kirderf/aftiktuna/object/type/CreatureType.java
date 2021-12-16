package me.kirderf.aftiktuna.object.type;

import me.kirderf.aftiktuna.object.entity.Stats;

public final class CreatureType extends ObjectType {
	private final Stats stats;
	
	public CreatureType(char symbol, String name, Stats stats) {
		super(symbol, name);
		this.stats = stats;
	}
	
	public Stats getStats() {
		return stats;
	}
}