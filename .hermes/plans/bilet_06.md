# Bilet 06: Çift Kanallı Arama ve Gölge Öğrenme Motoru Entegrasyonu

## Durum: AÇIK (Frontier — Uygulama Aşamasında)

## Amaç
`agent-reach-channels` içerisinde aramaları çift kanallı (statik arama + gölge semantik zihin haritası) yürütmek.

## Görevler:
1. `Channel::execute` çağrıldığında arka planda `agent-reach-graph` semantik genişletmesini çalıştırmak.
2. İlk aşamada semantik zihin haritası sonuçlarını gölgede (shadow mode) tutarak sadece statik sonuçları kullanıcıya sunmak.
3. Arama başarılı olduğunda semantik zihin haritasındaki ilgili düğümlerin ağırlıklarını arka planda otomatik güncellemek.
