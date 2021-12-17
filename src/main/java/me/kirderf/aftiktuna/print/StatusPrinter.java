package me.kirderf.aftiktuna.print;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.object.entity.Stats;
import me.kirderf.aftiktuna.object.type.ObjectType;
import me.kirderf.aftiktuna.object.type.WeaponType;

import java.io.PrintWriter;
import java.util.List;
import java.util.stream.Collectors;

public final class StatusPrinter {
	private final PrintWriter out;
	private final Crew crew;
	
	private int shownPoints = -1;
	//Printer for the aftik that is being controlled
	private AftikPrinter aftikPrinter;
	
	public StatusPrinter(PrintWriter out, Crew crew) {
		this.out = out;
		this.crew = crew;
	}
	
	public void printStatus(boolean fullStatus) {
		Aftik aftik = crew.getAftik();
		
		printCrewPoints(fullStatus);
		
		if (!(aftikPrinter != null && aftikPrinter.isForAftik(aftik))) {
			aftikPrinter = new AftikPrinter(aftik);
			aftikPrinter.printAftikStatus(true);
		} else
			aftikPrinter.printAftikStatus(fullStatus);
	}
	
	public void printCrewStatus() {
		printCrewPoints(true);
		out.printf("Crew:%n");
		for (Aftik aftik : crew.getCrewMembers()) {
			new AftikPrinter(aftik).printAftikProfile();
		}
	}
	
	public void printCrewPoints(boolean forcePrint) {
		if (forcePrint || crew.getPoints() != shownPoints) {
			out.printf("Crew points: %dp%n", crew.getPoints());
			shownPoints = crew.getPoints();
		}
	}
	
	final class AftikPrinter {
		private WeaponType shownWielded;
		private List<ObjectType> shownInventory;
		private float shownHealth;
		
		private final Aftik aftik;
		
		AftikPrinter(Aftik aftik) {
			this.aftik = aftik;
		}
		
		public boolean isForAftik(Aftik aftik) {
			return aftik == this.aftik;
		}
		
		public void printAftikProfile() {
			out.printf("%s (Aftik):%n", aftik.getName());
			printStats();
			printAftikStatus(true);
		}
		
		public void printAftikStatus(boolean fullStatus) {
			printHealth(fullStatus);
			printWieldedItem(fullStatus);
			printInventory(fullStatus);
		}
		
		private void printStats() {
			Stats stats = aftik.getStats();
			out.printf("%s%n", stats.displayText());
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
}