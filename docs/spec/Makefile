TEX = latexmk -pdf

IMGS = media/ProcedureKeyBytes.pdf media/ProcedureKeyLocation.pdf

.PHONY: all clean

all : BeakerSpec.pdf

BeakerSpec.pdf : BeakerSpec.tex $(IMGS)
	$(TEX) BeakerSpec.tex

%.pdf: %.svg
	inkscape --file=$< --export-area-page --without-gui --export-pdf=$@

clean:
	rm -f *.pdf *.aux *.toc *.dvi *.fdb_latexmk *.fls *.log *.out media/*.pdf
