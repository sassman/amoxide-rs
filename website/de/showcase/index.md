<script setup>
import { data } from '../../showcase/community.data'
</script>

# Community Showcase <VersionBadge v="0.4.0" />

Stöbere in Alias-Profilen, die von der Community geteilt wurden. Entdecke nützliche Profile, schaue dir die Aliase an und importiere sie mit einem einzigen Befehl.

Wenn du den Import-Befehl ausführst, zeigt `am` eine Zusammenfassung aller Aliase, bevor etwas übernommen wird — überprüfe sie sorgfältig, bevor du bestätigst.

<CommunityGallery :profiles="data" />

::: tip Möchtest du deine eigenen Profile teilen?
Schau in den [Beitragsleitfaden](./contribute), um zu erfahren, wie du deine einreichst.
:::
