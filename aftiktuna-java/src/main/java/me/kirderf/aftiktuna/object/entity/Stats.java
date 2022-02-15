package me.kirderf.aftiktuna.object.entity;

public record Stats(int strength, int endurance, int agility) {
	public String displayText() {
		return "Strength: %d   Endurance: %d   Agility: %d".formatted(strength, endurance, agility);
	}
}