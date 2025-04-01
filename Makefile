machine-1:
	./sushi --tip 10.0.0.3 --ulocal 192.168.97.3:8080 --uremote 192.168.97.2:8080

machine-2:
	./sushi --tip 10.0.0.2 --ulocal 192.168.97.2:8080 --uremote 192.168.97.3:8080
