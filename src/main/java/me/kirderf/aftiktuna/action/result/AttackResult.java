package me.kirderf.aftiktuna.action.result;

import me.kirderf.aftiktuna.object.entity.Entity;

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
