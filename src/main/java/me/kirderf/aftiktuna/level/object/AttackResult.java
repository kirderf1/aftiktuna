package me.kirderf.aftiktuna.level.object;

public record AttackResult(Entity attacked, Type type) {
	
	public enum Type {
		HIT,
		KILL,
		DODGE
	}
}
