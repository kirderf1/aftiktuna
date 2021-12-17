package me.kirderf.aftiktuna.object.type;

import me.kirderf.aftiktuna.object.entity.Stats;

public final class CreatureType extends ObjectType {
	private final Stats stats;
	private final String examineText;
	
	public CreatureType(char symbol, String name, Stats stats, String examineText) {
		super(symbol, name);
		this.stats = stats;
		this.examineText = examineText;
	}
	
	public Stats getStats() {
		return stats;
	}
	
	public String getExamineText() {
		return examineText;
	}
}