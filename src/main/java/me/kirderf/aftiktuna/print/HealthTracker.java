package me.kirderf.aftiktuna.print;

import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Entity;

import java.util.HashSet;
import java.util.Set;

public final class HealthTracker {
	private final Set<Entity> lowHealthTracked = new HashSet<>();
	
	public void tick(Aftik controlledAftik, ActionPrinter out) {
		lowHealthTracked.removeIf(entity -> entity.getPosition() == null || entity.getArea() != controlledAftik.getArea() || !isBadlyHurt(entity));
		
		controlledAftik.getArea().objectStream().flatMap(Entity.CAST.toStream()).forEach(entity -> {
			if (entity.isAlive() && isBadlyHurt(entity)) {
				if (lowHealthTracked.add(entity) && entity != controlledAftik)
					out.print("%s is badly hurt.", entity.getDisplayName(true, true));
			}
		});
	}
	
	private static boolean isBadlyHurt(Entity entity) {
		return entity.getHealth() < entity.getMaxHealth() * 0.5;
	}
}