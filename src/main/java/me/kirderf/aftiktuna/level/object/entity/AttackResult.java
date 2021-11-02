package me.kirderf.aftiktuna.level.object.entity;

public record AttackResult(Entity attacked, Type type) {
	
	public boolean isKill() {
		return attacked.isDead();
	}
	
	public enum Type {
		DIRECT_HIT,
		GRAZING_HIT,
		DODGE
	}
}
