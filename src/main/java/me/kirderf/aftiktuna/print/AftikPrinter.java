package me.kirderf.aftiktuna.print;

import me.kirderf.aftiktuna.object.ObjectType;
import me.kirderf.aftiktuna.object.WeaponType;
import me.kirderf.aftiktuna.object.entity.Aftik;

import java.io.PrintWriter;
import java.util.List;
import java.util.stream.Collectors;

final class AftikPrinter {
	private final PrintWriter out;
	private WeaponType shownWielded;
	private List<ObjectType> shownInventory;
	private float shownHealth;
	
	private final Aftik aftik;
	
	AftikPrinter(PrintWriter out, Aftik aftik) {
		this.out = out;
		this.aftik = aftik;
	}
	
	public boolean isForAftik(Aftik aftik) {
		return aftik == this.aftik;
	}
	
	public void printAftikWithName() {
		out.printf("%s (Aftik):%n", aftik.getName());
		printAftik(true);
	}
	
	public void printAftik(boolean fullStatus) {
		printHealth(fullStatus);
		printWieldedItem(fullStatus);
		printInventory(fullStatus);
	}
	
	private void printHealth(boolean forcePrint) {
		float health = aftik.getHealth();
		if (forcePrint || shownHealth != health) {
			final int barLength = 10;
			
			StringBuilder builder = new StringBuilder();
			for (int i = 0; i < barLength; i++) {
				builder.append(i * aftik.getMaxHealth() < barLength * health ? '#' : '.');
			}
			out.printf("Health: %s%n", builder);
			shownHealth = health;
		}
	}
	
	private void printWieldedItem(boolean forcePrint) {
		aftik.getWieldedItem().ifPresentOrElse(wielded -> {
			if (forcePrint || shownWielded != wielded)
				out.printf("Wielding: %s%n", wielded.capitalizedName());
			shownWielded = wielded;
		}, () -> {
			if (forcePrint || shownWielded != null)
				out.printf("Wielding: Nothing%n");
			shownWielded = null;
		});
	}
	
	private void printInventory(boolean forcePrint) {
		List<ObjectType> inventory = aftik.getInventory();
		if (forcePrint || !inventory.equals(shownInventory)) {
			if (!inventory.isEmpty()) {
				String itemList = inventory.stream().map(ObjectType::capitalizedName).collect(Collectors.joining(", "));
				out.printf("Inventory: %s%n", itemList);
			} else {
				out.printf("Inventory: Empty%n");
			}
			shownInventory = List.copyOf(inventory);
		}
	}
}
